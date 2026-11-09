//! Owned `.npy` v1.0 fixture interchange (§16.3 plane 1, fm-xb3).
//!
//! Structural fixtures against the Reference travel as NumPy `.npy` arrays:
//! the generation scripts (`scripts/gen_*.py`) emit them with `np.save`, and
//! the Gauntlet reads them here. This is a deliberately strict, deliberately
//! small reader/writer for the interchange subset the fixtures use:
//!
//! - format version 1.0 (2.0 headers are accepted on read),
//! - little-endian `<f8`, `<f4`, `<i8` dtypes only,
//! - C order only (`fortran_order: False`),
//! - 1 to 8 dimensions, element count checked exactly against the payload.
//!
//! Anything else is a precise, named error — fixtures are inputs, and a
//! malformed header must never become an allocation bomb (§16.5): the header
//! is capped, dimension counts are capped, and every size computation is
//! checked arithmetic.
//!
//! **The format is fnp-io's; the subset is this module's** (fm-sum). The
//! document parser and the header encoder are `fnp_io::read_npy_bytes` /
//! `fnp_io::write_npy_bytes` — the designated provider now that the
//! FrankenSuite is consumable from SUITE.lock (D1). There is exactly one npy
//! parser in the workspace, and it is not this one. What stays here is the
//! *policy*: the version whitelist, the dtype whitelist, C order, the
//! dimensionality cap, and the named [`NpyError`] vocabulary the Gauntlet
//! reports with.
//!
//! Two acceptance behaviours moved with the parser, and both loosen what a
//! *malformed* header may contain (well-formed `np.save` output is unaffected,
//! and `tests/npy_interchange.rs` keeps `write_npy(read_npy(bytes)) == bytes`
//! the law):
//!
//! - unknown dictionary keys are ignored by fnp-io rather than refused by
//!   name;
//! - redundant separators inside the shape tuple (`(1,,2)`) are skipped
//!   rather than refused.
//!
//! One diagnostic narrowed: fnp-io's payload verdict is exact but carries no
//! counts, so a shape/payload disagreement on *read* now surfaces as
//! [`NpyError::Truncated`] rather than [`NpyError::DataLength`].
//! [`NpyArray::new`] still reports `DataLength` with both counts, which is
//! where callers construct arrays. Recovering it on the read path needs an
//! upstream fnp-io change (counts in `ReadPayloadIncomplete`, or a
//! header-only parse entry point).

use std::fmt;

/// Hard cap on the declared header length. numpy v1.0 headers are u16-sized
/// anyway; v2.0 declares u32 and this cap is what keeps that honest. Bound to
/// fnp-io's own budget so the two can never drift apart.
const MAX_HEADER_LEN: usize = fnp_io::MAX_HEADER_BYTES;
/// Hard cap on dimensionality; fixture arrays are 1-D or 2-D in practice.
/// Narrower than fnp-io's rank budget on purpose — this is the interchange
/// subset, not the format.
const MAX_DIMS: usize = 8;

/// The element type of an interchange array.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DType {
    /// `<f8`: little-endian IEEE-754 double.
    F64,
    /// `<f4`: little-endian IEEE-754 single.
    F32,
    /// `<i8`: little-endian signed 64-bit integer.
    I64,
}

impl DType {
    /// The numpy descr string for this dtype.
    #[must_use]
    pub fn descr(self) -> &'static str {
        match self {
            Self::F64 => "<f8",
            Self::F32 => "<f4",
            Self::I64 => "<i8",
        }
    }

    /// Element size in bytes.
    #[must_use]
    pub fn size(self) -> usize {
        match self {
            Self::F64 | Self::I64 => 8,
            Self::F32 => 4,
        }
    }
}

/// The payload of an interchange array, in C (row-major) order.
#[derive(Clone, PartialEq, Debug)]
pub enum NpyData {
    /// `<f8` elements.
    F64(Vec<f64>),
    /// `<f4` elements.
    F32(Vec<f32>),
    /// `<i8` elements.
    I64(Vec<i64>),
}

impl NpyData {
    /// The dtype of this payload.
    #[must_use]
    pub fn dtype(&self) -> DType {
        match self {
            Self::F64(_) => DType::F64,
            Self::F32(_) => DType::F32,
            Self::I64(_) => DType::I64,
        }
    }

    /// Number of elements.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::F64(v) => v.len(),
            Self::F32(v) => v.len(),
            Self::I64(v) => v.len(),
        }
    }

    /// Whether the payload is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A decoded (or to-be-encoded) `.npy` array: shape plus C-order payload.
#[derive(Clone, PartialEq, Debug)]
pub struct NpyArray {
    /// The array shape; product equals the payload length.
    pub shape: Vec<usize>,
    /// The payload.
    pub data: NpyData,
}

impl NpyArray {
    /// Construct an array, checking that the shape's element count matches
    /// the payload length and respects the dimensionality cap.
    ///
    /// # Errors
    /// [`NpyError::TooManyDims`], [`NpyError::Overflow`], or
    /// [`NpyError::DataLength`] when shape and payload disagree.
    pub fn new(shape: Vec<usize>, data: NpyData) -> Result<Self, NpyError> {
        let count = element_count(&shape)?;
        if count != data.len() {
            return Err(NpyError::DataLength {
                expected: count,
                actual: data.len(),
            });
        }
        Ok(Self { shape, data })
    }

    /// View the payload as `&[f64]`, if that is its dtype.
    #[must_use]
    pub fn as_f64(&self) -> Option<&[f64]> {
        match &self.data {
            NpyData::F64(v) => Some(v),
            _ => None,
        }
    }

    /// View the payload as `&[f32]`, if that is its dtype.
    #[must_use]
    pub fn as_f32(&self) -> Option<&[f32]> {
        match &self.data {
            NpyData::F32(v) => Some(v),
            _ => None,
        }
    }

    /// View the payload as `&[i64]`, if that is its dtype.
    #[must_use]
    pub fn as_i64(&self) -> Option<&[i64]> {
        match &self.data {
            NpyData::I64(v) => Some(v),
            _ => None,
        }
    }

    /// Interpret a `(n, 3)` f64 array as a point run — the shape constructor
    /// point-array fixtures use.
    ///
    /// # Errors
    /// [`NpyError::NotPoints`] unless the array is exactly 2-D with a
    /// trailing dimension of 3 and dtype `<f8`.
    pub fn to_points(&self) -> Result<Vec<[f64; 3]>, NpyError> {
        let Some(flat) = self.as_f64() else {
            return Err(NpyError::NotPoints {
                detail: format!("dtype is {:?}, expected <f8", self.data.dtype()),
            });
        };
        let &[_, 3] = self.shape.as_slice() else {
            return Err(NpyError::NotPoints {
                detail: format!("shape is {:?}, expected (n, 3)", self.shape),
            });
        };
        let (chunks, _rem) = flat.as_chunks::<3>();
        Ok(chunks.to_vec())
    }
}

/// A precise interchange failure.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NpyError {
    /// The file is shorter than the structure it declares.
    Truncated {
        /// What was being read when the bytes ran out.
        reading: &'static str,
    },
    /// The six-byte magic is absent.
    BadMagic,
    /// A format version other than 1.0 / 2.0.
    UnsupportedVersion {
        /// Declared major.
        major: u8,
        /// Declared minor.
        minor: u8,
    },
    /// The declared header length exceeds the cap.
    HeaderTooLarge {
        /// Declared length.
        len: usize,
        /// The cap (`MAX_HEADER_LEN`).
        max: usize,
    },
    /// The header is not ASCII or its dict does not parse.
    Header {
        /// What was wrong.
        detail: String,
    },
    /// A descr outside the interchange subset.
    UnsupportedDescr {
        /// The descr string found.
        descr: String,
    },
    /// `fortran_order: True` — the interchange subset is C order only.
    FortranOrder,
    /// More dimensions than the cap.
    TooManyDims {
        /// Declared dimensionality.
        dims: usize,
        /// The cap (`MAX_DIMS`).
        max: usize,
    },
    /// Shape-product or size arithmetic overflowed.
    Overflow,
    /// Payload length disagrees with the declared shape.
    DataLength {
        /// Elements the shape declares.
        expected: usize,
        /// Elements the payload holds.
        actual: usize,
    },
    /// [`NpyArray::to_points`] on an array that is not `(n, 3)` `<f8`.
    NotPoints {
        /// What shape/dtype was found instead.
        detail: String,
    },
}

impl fmt::Display for NpyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { reading } => write!(f, "truncated .npy while reading {reading}"),
            Self::BadMagic => write!(f, "not a .npy file (bad magic)"),
            Self::UnsupportedVersion { major, minor } => {
                write!(f, "unsupported .npy format version {major}.{minor}")
            }
            Self::HeaderTooLarge { len, max } => {
                write!(f, ".npy header length {len} exceeds cap {max}")
            }
            Self::Header { detail } => write!(f, "malformed .npy header: {detail}"),
            Self::UnsupportedDescr { descr } => write!(
                f,
                "unsupported .npy descr {descr:?}: interchange subset is <f8, <f4, <i8"
            ),
            Self::FortranOrder => {
                write!(
                    f,
                    ".npy declares fortran_order: interchange subset is C order only"
                )
            }
            Self::TooManyDims { dims, max } => {
                write!(f, ".npy declares {dims} dimensions, cap is {max}")
            }
            Self::Overflow => write!(f, ".npy size arithmetic overflowed"),
            Self::DataLength { expected, actual } => write!(
                f,
                ".npy payload holds {actual} elements but the shape declares {expected}"
            ),
            Self::NotPoints { detail } => {
                write!(f, ".npy array is not a (n, 3) <f8 point run: {detail}")
            }
        }
    }
}

impl std::error::Error for NpyError {}

fn element_count(shape: &[usize]) -> Result<usize, NpyError> {
    if shape.is_empty() || shape.len() > MAX_DIMS {
        return Err(NpyError::TooManyDims {
            dims: shape.len(),
            max: MAX_DIMS,
        });
    }
    shape
        .iter()
        .try_fold(1usize, |acc, &d| acc.checked_mul(d))
        .ok_or(NpyError::Overflow)
}

/// Map the interchange subset onto fnp-io's dtype model.
impl From<DType> for fnp_io::IOSupportedDType {
    fn from(dtype: DType) -> Self {
        match dtype {
            DType::F64 => Self::F64,
            DType::F32 => Self::F32,
            DType::I64 => Self::I64,
        }
    }
}

impl DType {
    /// The interchange subset's whitelist, applied to whatever fnp-io decoded.
    /// Everything fnp-io understands but this module does not accept is named
    /// by its descr, not silently widened.
    fn from_io(dtype: fnp_io::IOSupportedDType) -> Result<Self, NpyError> {
        match dtype {
            fnp_io::IOSupportedDType::F64 => Ok(Self::F64),
            fnp_io::IOSupportedDType::F32 => Ok(Self::F32),
            fnp_io::IOSupportedDType::I64 => Ok(Self::I64),
            other => Err(NpyError::UnsupportedDescr {
                descr: other.descr(),
            }),
        }
    }
}

/// fnp-io's verdict in this module's vocabulary.
///
/// The framing errors ([`NpyError::BadMagic`], [`NpyError::UnsupportedVersion`],
/// [`NpyError::HeaderTooLarge`]) are decided by [`read_npy`] before delegating,
/// so they never arrive here.
fn map_io_error(error: fnp_io::IOError) -> NpyError {
    match error {
        fnp_io::IOError::MagicInvalid => NpyError::BadMagic,
        fnp_io::IOError::DTypeDescriptorInvalid => NpyError::Header {
            detail: "descr is not a NumPy dtype descriptor".to_string(),
        },
        // fnp-io's payload verdict is exact ("payload bytes must exactly match
        // expected shape/dtype footprint") but carries no counts, so the read
        // path can no longer name expected/actual the way `NpyArray::new`
        // still does. See the module docs.
        fnp_io::IOError::ReadPayloadIncomplete(_) => NpyError::Truncated { reading: "payload" },
        other => NpyError::Header {
            detail: other.to_string(),
        },
    }
}

/// Read this module's framing policy off the preamble: the magic, the format
/// version whitelist, and the declared header length against the cap.
///
/// This is deliberately *not* a second parser — it decodes no dictionary and
/// no shape. It exists because the interchange subset admits a narrower set of
/// versions than fnp-io does (which also reads 3.0), and because §16.5 wants
/// the header bound named before anything is decoded.
fn read_framing(bytes: &[u8]) -> Result<(), NpyError> {
    let magic = bytes
        .first_chunk::<6>()
        .ok_or(NpyError::Truncated { reading: "magic" })?;
    if magic != &fnp_io::NPY_MAGIC_PREFIX {
        return Err(NpyError::BadMagic);
    }
    let &[major, minor] = bytes
        .get(6..8)
        .ok_or(NpyError::Truncated { reading: "version" })?
    else {
        // get(6..8) yields exactly two bytes; keep the reader panic-free anyway.
        return Err(NpyError::Truncated { reading: "version" });
    };
    let header_len = match (major, minor) {
        (1, 0) => {
            let len = bytes.get(8..10).ok_or(NpyError::Truncated {
                reading: "header length",
            })?;
            usize::from(u16::from_le_bytes([len[0], len[1]]))
        }
        (2, 0) => {
            let len = bytes.get(8..12).ok_or(NpyError::Truncated {
                reading: "header length",
            })?;
            let len = u32::from_le_bytes([len[0], len[1], len[2], len[3]]);
            usize::try_from(len).map_err(|_| NpyError::Overflow)?
        }
        _ => return Err(NpyError::UnsupportedVersion { major, minor }),
    };
    if header_len > MAX_HEADER_LEN {
        return Err(NpyError::HeaderTooLarge {
            len: header_len,
            max: MAX_HEADER_LEN,
        });
    }
    Ok(())
}

/// Decode a `.npy` document from `bytes`.
///
/// The document is parsed by fnp-io; this function owns the interchange
/// subset's policy on top of it — version whitelist, C order, the `<f8`/`<f4`/
/// `<i8` dtype whitelist, the dimensionality cap, and the exact element count.
///
/// # Errors
/// A precise [`NpyError`] naming the first thing wrong with the document.
pub fn read_npy(bytes: &[u8]) -> Result<NpyArray, NpyError> {
    read_framing(bytes)?;
    let decoded = fnp_io::read_npy_bytes(bytes, false).map_err(map_io_error)?;
    if decoded.header.fortran_order {
        return Err(NpyError::FortranOrder);
    }
    let dtype = DType::from_io(decoded.header.descr)?;
    let shape = decoded.header.shape;
    // The dimensionality cap is this module's, not fnp-io's (which admits 32).
    // fnp-io has already bounded the header itself, so the shape vector it
    // hands back is bounded by that budget before this refuses it.
    let count = element_count(&shape)?;
    let payload: &[u8] = &decoded.payload;
    // fnp-io validated `payload.len() == count * item_size` before returning.
    debug_assert_eq!(payload.len(), count * dtype.size());
    let data = match dtype {
        DType::F64 => {
            let (chunks, _rem) = payload.as_chunks::<8>();
            NpyData::F64(chunks.iter().map(|c| f64::from_le_bytes(*c)).collect())
        }
        DType::F32 => {
            let (chunks, _rem) = payload.as_chunks::<4>();
            NpyData::F32(chunks.iter().map(|c| f32::from_le_bytes(*c)).collect())
        }
        DType::I64 => {
            let (chunks, _rem) = payload.as_chunks::<8>();
            NpyData::I64(chunks.iter().map(|c| i64::from_le_bytes(*c)).collect())
        }
    };
    NpyArray::new(shape, data)
}

/// Encode `array` as a `.npy` v1.0 document, byte-compatible with what
/// `np.save` produces for the same array.
///
/// The bytes come from fnp-io's writer, whose header encoding — numpy's key
/// order, spacing, `, }` terminator, and 64-byte alignment padding — is the
/// same construction this module used to perform itself; `tests/npy_interchange.rs`
/// keeps `write_npy(read_npy(bytes)) == bytes` the law against real `np.save`
/// output.
///
/// # Panics
/// If `array`'s shape and payload disagree. [`NpyArray::new`] proves they
/// agree; the fields are public, so a hand-built `NpyArray` can violate it,
/// and this refuses to emit a corrupt document rather than emitting one
/// silently.
#[must_use]
pub fn write_npy(array: &NpyArray) -> Vec<u8> {
    let dtype = array.data.dtype();
    let mut payload = Vec::with_capacity(array.data.len() * dtype.size());
    match &array.data {
        NpyData::F64(v) => {
            for x in v {
                payload.extend_from_slice(&x.to_le_bytes());
            }
        }
        NpyData::F32(v) => {
            for x in v {
                payload.extend_from_slice(&x.to_le_bytes());
            }
        }
        NpyData::I64(v) => {
            for x in v {
                payload.extend_from_slice(&x.to_le_bytes());
            }
        }
    }
    let header = fnp_io::NpyHeader {
        shape: array.shape.clone(),
        fortran_order: false,
        descr: dtype.into(),
    };
    fnp_io::write_npy_bytes(&header, &payload, false)
        .expect("NpyArray::new proves the shape and payload agree for a non-object dtype")
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Build a v1.0 document around an arbitrary header dict. Test-only
    /// construction so the reader under test does all the parsing.
    fn doc(dict: &str, payload: &[u8]) -> Vec<u8> {
        let unpadded = 10 + dict.len() + 1;
        let padding = (64 - unpadded % 64) % 64;
        let header_len = dict.len() + padding + 1;
        let mut out = Vec::new();
        out.extend_from_slice(&fnp_io::NPY_MAGIC_PREFIX);
        out.extend_from_slice(&[1, 0]);
        out.extend_from_slice(
            &u16::try_from(header_len)
                .expect("test header fits u16")
                .to_le_bytes(),
        );
        out.extend_from_slice(dict.as_bytes());
        out.resize(out.len() + padding, b' ');
        out.push(b'\n');
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn round_trip_f64_2d() {
        let a = NpyArray::new(
            vec![2, 3],
            NpyData::F64(vec![0.0, -0.0, 1.5, -2.25, f64::MAX, f64::MIN_POSITIVE]),
        )
        .unwrap();
        let bytes = write_npy(&a);
        // Header block is 64-byte aligned and v1.0 — fnp-io's writer reproduces
        // the same construction this module used to perform itself.
        assert_eq!(&bytes[..6], &fnp_io::NPY_MAGIC_PREFIX);
        assert_eq!(&bytes[6..8], &[1, 0]);
        let hlen = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        assert_eq!((10 + hlen) % 64, 0);
        assert_eq!(bytes[10 + hlen - 1], b'\n');
        let b = read_npy(&bytes).unwrap();
        assert_eq!(a, b);
        // −0.0 survives bit-exactly (interchange is bytes, not values).
        assert_eq!(b.as_f64().unwrap()[1].to_bits(), (-0.0f64).to_bits());
    }

    #[test]
    fn round_trip_f32_and_i64_1d() {
        for data in [
            NpyData::F32(vec![1.0, -3.5, f32::EPSILON]),
            NpyData::I64(vec![i64::MIN, -1, 0, i64::MAX]),
        ] {
            let n = data.len();
            let a = NpyArray::new(vec![n], data).unwrap();
            assert_eq!(read_npy(&write_npy(&a)).unwrap(), a);
        }
    }

    /// The exact byte sequence numpy's own writer emits for the canonical
    /// header — the construction `tests/npy_interchange.rs` locks against real
    /// `np.save` output, asserted here at the unit level so a drift in fnp-io's
    /// encoder is named here first.
    #[test]
    fn header_bytes_are_numpys_canonical_form() {
        let a = NpyArray::new(vec![2, 3], NpyData::F64(vec![0.0; 6])).unwrap();
        let bytes = write_npy(&a);
        let hlen = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        let header = std::str::from_utf8(&bytes[10..10 + hlen]).unwrap();
        assert_eq!(
            header.trim_end(),
            "{'descr': '<f8', 'fortran_order': False, 'shape': (2, 3), }"
        );
        let one = write_npy(&NpyArray::new(vec![1], NpyData::F32(vec![0.0])).unwrap());
        let hlen = usize::from(u16::from_le_bytes([one[8], one[9]]));
        assert_eq!(
            std::str::from_utf8(&one[10..10 + hlen]).unwrap().trim_end(),
            "{'descr': '<f4', 'fortran_order': False, 'shape': (1,), }"
        );
    }

    #[test]
    fn header_keys_parse_in_any_order() {
        let bytes = doc(
            "{'shape': (4, 3), 'fortran_order': False, 'descr': '<f4'}",
            &[0u8; 4 * 3 * 4],
        );
        let array = read_npy(&bytes).expect("key order is not significant");
        assert_eq!(array.shape, vec![4, 3]);
        assert_eq!(array.data.dtype(), DType::F32);
    }

    #[test]
    fn named_errors_for_bad_documents() {
        assert_eq!(read_npy(b"not npy").unwrap_err(), NpyError::BadMagic);

        // A short payload is refused. fnp-io's verdict is exact but carries no
        // counts, so this surfaces as Truncated rather than DataLength.
        let a = NpyArray::new(vec![1], NpyData::F64(vec![1.0])).unwrap();
        let mut bytes = write_npy(&a);
        bytes.truncate(bytes.len() - 4);
        assert_eq!(
            read_npy(&bytes).unwrap_err(),
            NpyError::Truncated { reading: "payload" }
        );

        // Fortran order is refused by name: fnp-io parses it, this module's
        // policy declines it.
        let fortran = doc(
            "{'descr': '<f8', 'fortran_order': True, 'shape': (1,), }",
            &[0u8; 8],
        );
        assert_eq!(read_npy(&fortran).unwrap_err(), NpyError::FortranOrder);

        // A dtype fnp-io understands but the interchange subset does not is
        // named by its descr.
        let narrow = doc(
            "{'descr': '<i4', 'fortran_order': False, 'shape': (1,), }",
            &[0u8; 4],
        );
        assert_eq!(
            read_npy(&narrow).unwrap_err(),
            NpyError::UnsupportedDescr {
                descr: "<i4".to_string()
            }
        );
    }

    /// This module's version whitelist is narrower than fnp-io's, which also
    /// reads 3.0. The framing check is what keeps it narrow.
    #[test]
    fn version_whitelist_is_this_modules() {
        let mut v3 = Vec::from(fnp_io::NPY_MAGIC_PREFIX);
        v3.extend_from_slice(&[3, 0]);
        v3.extend_from_slice(&64u32.to_le_bytes());
        assert_eq!(
            read_npy(&v3).unwrap_err(),
            NpyError::UnsupportedVersion { major: 3, minor: 0 }
        );
    }

    /// §16.5: the declared header length is named against the cap before any
    /// dictionary is decoded.
    #[test]
    fn oversized_header_declaration_is_capped_by_name() {
        let mut huge = Vec::from(fnp_io::NPY_MAGIC_PREFIX);
        huge.extend_from_slice(&[2, 0]);
        let declared = u32::try_from(MAX_HEADER_LEN + 1).unwrap();
        huge.extend_from_slice(&declared.to_le_bytes());
        assert_eq!(
            read_npy(&huge).unwrap_err(),
            NpyError::HeaderTooLarge {
                len: MAX_HEADER_LEN + 1,
                max: MAX_HEADER_LEN
            }
        );
    }

    #[test]
    fn to_points_requires_n_by_3_f64() {
        let pts = NpyArray::new(vec![2, 3], NpyData::F64((0..6).map(f64::from).collect())).unwrap();
        assert_eq!(
            pts.to_points().unwrap(),
            vec![[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]
        );
        let flat = NpyArray::new(vec![6], NpyData::F64((0..6).map(f64::from).collect())).unwrap();
        assert!(matches!(flat.to_points(), Err(NpyError::NotPoints { .. })));
    }

    #[test]
    fn shape_payload_disagreement_is_refused() {
        assert!(matches!(
            NpyArray::new(vec![2, 3], NpyData::F64(vec![0.0; 5])),
            Err(NpyError::DataLength {
                expected: 6,
                actual: 5
            })
        ));
    }

    /// The dimensionality cap is this module's own (8), applied to whatever
    /// fnp-io hands back. Beyond fnp-io's own rank budget the shared parser
    /// refuses first, which is a `Header` diagnostic rather than this one.
    #[test]
    fn dimension_cap_is_this_modules_policy() {
        let nine = "1, ".repeat(8) + "1";
        let bytes = doc(
            &format!("{{'descr': '<f8', 'fortran_order': False, 'shape': ({nine}), }}"),
            &[0u8; 8],
        );
        let error = read_npy(&bytes).expect_err("nine dimensions exceed this module's cap");
        assert_eq!(
            error,
            NpyError::TooManyDims {
                dims: 9,
                max: MAX_DIMS
            }
        );
        assert!(
            error.to_string().len() < 128,
            "dimension-cap diagnostic must stay bounded: {error}"
        );

        let far_past = "1, ".repeat(64) + "1";
        let bytes = doc(
            &format!("{{'descr': '<f8', 'fortran_order': False, 'shape': ({far_past}), }}"),
            &[0u8; 8],
        );
        assert!(matches!(
            read_npy(&bytes).unwrap_err(),
            NpyError::Header { .. }
        ));
    }

    /// The two acceptance behaviours that moved to the shared parser, pinned
    /// so the loosening stays visible and any future drift is caught here.
    /// Neither affects well-formed `np.save` output.
    #[test]
    fn shared_parser_acceptance_is_pinned() {
        // Unknown keys are ignored rather than refused by name.
        let extra_key = doc(
            "{'descr': '<f8', 'fortran_order': False, 'shape': (1,), 'x': 1, }",
            &[0u8; 8],
        );
        assert_eq!(
            read_npy(&extra_key)
                .expect("fnp-io ignores unknown keys")
                .shape,
            vec![1]
        );

        // Redundant shape separators are skipped rather than refused.
        let doubled = doc(
            "{'descr': '<f8', 'fortran_order': False, 'shape': (1,,2), }",
            &[0u8; 16],
        );
        assert_eq!(
            read_npy(&doubled)
                .expect("fnp-io skips empty shape parts")
                .shape,
            vec![1, 2]
        );

        // A singleton tuple still requires its trailing comma.
        let bare = doc(
            "{'descr': '<f8', 'fortran_order': False, 'shape': (1), }",
            &[0u8; 8],
        );
        assert!(matches!(
            read_npy(&bare).unwrap_err(),
            NpyError::Header { .. }
        ));
    }
}
