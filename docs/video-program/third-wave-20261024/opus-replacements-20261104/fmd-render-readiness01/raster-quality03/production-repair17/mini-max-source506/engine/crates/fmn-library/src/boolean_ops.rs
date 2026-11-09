//! The Reference's boolean-op mobjects (fm-6l6): `Union`, `Difference`,
//! `Intersection`, and `Exclusion` as thin wrappers over Chisel's certified
//! path boolean ([`fmn_geom::boolean`]), which displaced skia-pathops.
//!
//! Reference semantics kept (D5): each operand contributes its whole
//! `family_members_with_points` geometry merged into one path; `Union`/
//! `Intersection`/`Exclusion` require at least two operands (the
//! `ValueError` parity case); `Difference` takes exactly subject and clip;
//! the result is a fresh default-styled `VMobject` — the Reference's
//! constructor discards operand styles, and so do we. Multi-operand ops
//! fold left-to-right through the binary kernel (the Reference's
//! `Intersection`/`Exclusion` fold the same way; its n-ary skia `union` is
//! point-set identical to a fold).
//!
//! Routing is surfaced, never inferred: every kernel call's
//! [`BooleanRoute`] is recorded in [`BooleanBuild::routes`] in call order,
//! so a caller (and the tests) can assert WHICH implementation ran. An
//! unsupported overlap class rendering through the certified
//! [`BooleanRoute::FlattenClip`] fallback is correct topology, honestly
//! labeled. Degenerate point-set identities (empty operands) are resolved
//! without a kernel call and simply record no route.
//!
//! Empty-operand semantics are the point-set identities skia also
//! produces: empty is the identity for union and exclusion and the
//! annihilator for intersection; an empty clip leaves the difference
//! subject unchanged and an empty subject differences to nothing.

use fmn_geom::boolean::{
    BooleanError, BooleanOperation, BooleanOptions, BooleanRoute, path_boolean,
};
use fmn_geom::{GeomError, QuadPath};

use crate::vmobject::VMobject;

/// A built boolean mobject plus the routing audit trail.
///
/// `routes` records one [`BooleanRoute`] per kernel call, in call order —
/// the fold order documented on each constructor. A degenerate identity
/// resolved without a kernel call records nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct BooleanBuild {
    mobject: VMobject,
    routes: Vec<BooleanRoute>,
}

impl BooleanBuild {
    /// The boolean result as a fresh default-styled mobject.
    #[must_use]
    pub fn into_mobject(self) -> VMobject {
        self.mobject
    }

    /// The boolean result.
    #[must_use]
    pub fn mobject(&self) -> &VMobject {
        &self.mobject
    }

    /// Which implementation produced each kernel call's contribution, in
    /// fold order.
    #[must_use]
    pub fn routes(&self) -> &[BooleanRoute] {
        &self.routes
    }
}

/// Typed failures from the boolean mobject constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooleanMobjectError {
    /// Fewer than two operands reached an n-ary constructor — the
    /// Reference's `ValueError: At least 2 mobjects needed for …`.
    NeedTwoOperands(&'static str),
    /// Operand geometry failed to load.
    Geometry(GeomError),
    /// The certified kernel refused the operation.
    Boolean(BooleanError),
}

impl From<GeomError> for BooleanMobjectError {
    fn from(err: GeomError) -> Self {
        Self::Geometry(err)
    }
}

impl From<BooleanError> for BooleanMobjectError {
    fn from(err: BooleanError) -> Self {
        Self::Boolean(err)
    }
}

impl std::fmt::Display for BooleanMobjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeedTwoOperands(op) => {
                write!(f, "At least 2 mobjects needed for {op}.")
            }
            Self::Geometry(err) => write!(f, "boolean operand geometry: {err}"),
            Self::Boolean(err) => write!(f, "boolean kernel: {err}"),
        }
    }
}

impl std::error::Error for BooleanMobjectError {}

/// One operand's whole family geometry, the Reference's
/// `family_members_with_points` merge: self plus every descendant that
/// carries points, concatenated in family order.
fn family_path(vmob: &VMobject) -> Result<QuadPath, GeomError> {
    let mut merged = QuadPath::new();
    let mut stack = vec![vmob];
    while let Some(current) = stack.pop() {
        // Depth-first in reverse so children append in family order.
        for child in current.children().iter().rev() {
            stack.push(child);
        }
        if current.points().is_empty() {
            continue;
        }
        for subpath in current.path()?.subpaths() {
            merged.add_subpath(subpath)?;
        }
    }
    Ok(merged)
}

/// Fold the binary kernel left-to-right over non-empty operand paths.
///
/// `identity_when_all_empty` mirrors the two point-set degeneracies: union
/// and exclusion of nothing are nothing, and so is intersection of
/// nothing once an annihilator is present (handled by the caller).
fn fold(
    paths: Vec<QuadPath>,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> Result<(QuadPath, Vec<BooleanRoute>), BooleanError> {
    let mut routes = Vec::new();
    let mut non_empty: Vec<QuadPath> = paths.into_iter().filter(|path| path.has_points()).collect();
    if non_empty.len() <= 1 {
        return Ok((non_empty.pop().unwrap_or_default(), routes));
    }
    let mut accumulator = non_empty.remove(0);
    for operand in &non_empty {
        let result = path_boolean(&accumulator, operand, operation, options)?;
        routes.push(result.route);
        accumulator = result.path;
    }
    Ok((accumulator, routes))
}

/// The Reference's `Union(*vmobjects)`: points contained by any operand.
///
/// At least two operands are required. The fold runs one kernel call per
/// additional operand.
pub fn union(vmobjects: &[VMobject]) -> Result<BooleanBuild, BooleanMobjectError> {
    union_with_options(vmobjects, BooleanOptions::default())
}

/// [`union`] with explicit kernel options (budgets, tolerance, fill rules).
pub fn union_with_options(
    vmobjects: &[VMobject],
    options: BooleanOptions,
) -> Result<BooleanBuild, BooleanMobjectError> {
    if vmobjects.len() < 2 {
        return Err(BooleanMobjectError::NeedTwoOperands("Union"));
    }
    let mut paths = Vec::with_capacity(vmobjects.len());
    for vmob in vmobjects {
        paths.push(family_path(vmob)?);
    }
    let (path, routes) = fold(paths, BooleanOperation::Union, options)?;
    Ok(BooleanBuild {
        mobject: VMobject::from_path(&path),
        routes,
    })
}

/// The Reference's `Difference(subject, clip)`: points in the subject and
/// not in the clip.
pub fn difference(
    subject: &VMobject,
    clip: &VMobject,
) -> Result<BooleanBuild, BooleanMobjectError> {
    difference_with_options(subject, clip, BooleanOptions::default())
}

/// [`difference`] with explicit kernel options.
pub fn difference_with_options(
    subject: &VMobject,
    clip: &VMobject,
    options: BooleanOptions,
) -> Result<BooleanBuild, BooleanMobjectError> {
    let subject_path = family_path(subject)?;
    let clip_path = family_path(clip)?;
    // Point-set degeneracies, resolved without a kernel call.
    let (path, routes) = match (!subject_path.has_points(), !clip_path.has_points()) {
        (true, _) => (QuadPath::new(), Vec::new()),
        (false, true) => (subject_path, Vec::new()),
        (false, false) => {
            let result = path_boolean(
                &subject_path,
                &clip_path,
                BooleanOperation::Difference,
                options,
            )?;
            (result.path, vec![result.route])
        }
    };
    Ok(BooleanBuild {
        mobject: VMobject::from_path(&path),
        routes,
    })
}

/// The Reference's `Intersection(*vmobjects)`: points contained by every
/// operand, folded left-to-right exactly as the Reference folds.
pub fn intersection(vmobjects: &[VMobject]) -> Result<BooleanBuild, BooleanMobjectError> {
    intersection_with_options(vmobjects, BooleanOptions::default())
}

/// [`intersection`] with explicit kernel options.
pub fn intersection_with_options(
    vmobjects: &[VMobject],
    options: BooleanOptions,
) -> Result<BooleanBuild, BooleanMobjectError> {
    if vmobjects.len() < 2 {
        return Err(BooleanMobjectError::NeedTwoOperands("Intersection"));
    }
    let mut paths = Vec::with_capacity(vmobjects.len());
    for vmob in vmobjects {
        paths.push(family_path(vmob)?);
    }
    // The annihilator: any empty operand makes the intersection empty
    // without a kernel call.
    if paths.iter().any(|path| !path.has_points()) {
        return Ok(BooleanBuild {
            mobject: VMobject::new(),
            routes: Vec::new(),
        });
    }
    let (path, routes) = fold(paths, BooleanOperation::Intersection, options)?;
    Ok(BooleanBuild {
        mobject: VMobject::from_path(&path),
        routes,
    })
}

/// The Reference's `Exclusion(*vmobjects)`: points contained by an odd
/// number of operands (two-operand xor, folded left-to-right).
pub fn exclusion(vmobjects: &[VMobject]) -> Result<BooleanBuild, BooleanMobjectError> {
    exclusion_with_options(vmobjects, BooleanOptions::default())
}

/// [`exclusion`] with explicit kernel options.
pub fn exclusion_with_options(
    vmobjects: &[VMobject],
    options: BooleanOptions,
) -> Result<BooleanBuild, BooleanMobjectError> {
    if vmobjects.len() < 2 {
        return Err(BooleanMobjectError::NeedTwoOperands("Exclusion"));
    }
    let mut paths = Vec::with_capacity(vmobjects.len());
    for vmob in vmobjects {
        paths.push(family_path(vmob)?);
    }
    let (path, routes) = fold(paths, BooleanOperation::Exclusion, options)?;
    Ok(BooleanBuild {
        mobject: VMobject::from_path(&path),
        routes,
    })
}
