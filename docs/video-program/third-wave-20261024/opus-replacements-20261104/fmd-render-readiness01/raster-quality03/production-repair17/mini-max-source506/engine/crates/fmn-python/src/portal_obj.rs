//! Material-aware OBJ import through Atlas, with host-owned asset resolution.
//! Parsed documents and MTL tables are immutable. No path is opened here.
use std::collections::BTreeMap;

use fmn_library::obj_materials::{ObjAssetLimits, ObjDocument, ObjMaterial, parse_mtl};
use fmn_mobject::ImageResource;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

use super::raster::RasterImage;
use crate::{PyValueError, native_shell_specs};

const MAX_TEXTURE_BYTES: usize = 256 * 1024 * 1024;

#[pyclass(frozen, name = "_ObjDocument")]
struct Document {
    value: ObjDocument,
}

#[pymethods]
impl Document {
    #[new]
    fn new(py: Python<'_>, source: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let bytes = source.as_bytes();
        py.detach(|| {
            ObjDocument::parse(bytes)
                .map(|value| Self { value })
                .map_err(|error| PyValueError::new_err(error.to_string()))
        })
    }

    #[getter]
    fn libraries(&self) -> Vec<String> {
        self.value.libraries().to_vec()
    }

    #[getter]
    fn material_names(&self) -> Vec<String> {
        self.value.material_names().into_iter().collect()
    }

    #[getter]
    fn part_materials(&self) -> Vec<Option<String>> {
        self.value
            .runs()
            .iter()
            .map(|run| run.material.clone())
            .collect()
    }
}

#[pyclass(frozen, name = "_ObjMaterials")]
struct Materials {
    value: BTreeMap<String, ObjMaterial>,
}

#[pymethods]
impl Materials {
    #[new]
    fn new(py: Python<'_>, source: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let bytes = source.as_bytes();
        py.detach(|| {
            parse_mtl(bytes, &ObjAssetLimits::default())
                .map(|value| Self { value })
                .map_err(|error| PyValueError::new_err(error.to_string()))
        })
    }

    /// Resolve images relative to the MTL which first defines the material.
    #[getter]
    fn entries(&self) -> Vec<(String, Option<String>)> {
        self.value
            .iter()
            .map(|(name, value)| (name.clone(), value.texture.clone()))
            .collect()
    }
}

/// Return fully prepared native child specs, without changing a destination
/// proxy. The host installs these children only after all inputs are admitted.
#[pyfunction(signature = (document, libraries, images, factory, height=3.0))]
fn _build_obj_parts<'py>(
    py: Python<'py>,
    document: &Document,
    libraries: &Bound<'py, PyList>,
    images: &Bound<'py, PyDict>,
    factory: &Bound<'py, PyAny>,
    height: f64,
) -> PyResult<Bound<'py, PyList>> {
    let limits = ObjAssetLimits::default();
    if libraries.len() > limits.max_materials || images.len() > limits.max_materials {
        return Err(PyValueError::new_err(
            "OBJ material input count budget exceeded",
        ));
    }
    let mut materials = BTreeMap::new();
    for library in libraries.iter() {
        let library = library.cast::<Materials>()?.try_borrow()?;
        for (name, material) in &library.value {
            if !materials.contains_key(name) && materials.len() >= limits.max_materials {
                return Err(PyValueError::new_err(
                    "OBJ aggregate material count budget exceeded",
                ));
            }
            // File order is authoritative, never map iteration or I/O timing.
            materials
                .entry(name.clone())
                .or_insert_with(|| material.clone());
        }
    }
    let mut decoded = BTreeMap::<String, ImageResource>::new();
    let mut byte_count = 0usize;
    let used = document.value.material_names();
    for (key, value) in images.iter() {
        let name = key.extract::<String>()?;
        if !used.contains(&name) {
            return Err(PyValueError::new_err(
                "an OBJ image names an unused material",
            ));
        }
        let image = value.cast::<RasterImage>()?.try_borrow()?;
        // A conservative aggregate bound includes each material binding,
        // even when two bindings share the same immutable decoded allocation.
        byte_count = byte_count
            .checked_add(image.resource.pixels().len())
            .filter(|&size| size <= MAX_TEXTURE_BYTES)
            .ok_or_else(|| PyValueError::new_err("OBJ bound textures exceed the 256 MiB budget"))?;
        decoded.insert(name, image.resource.clone());
    }
    let tree = py
        .detach(|| document.value.to_mobject(height, &materials, &decoded))
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    native_shell_specs(py, factory, tree.submobjects)
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Document>()?;
    module.add_class::<Materials>()?;
    module.add_function(wrap_pyfunction!(_build_obj_parts, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_obj_material_acceptance() {
        crate::with_python_test_module("OBJ materials", |py, _module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/obj_materials.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native multi-material OBJ loading, pixels and ownership");
        });
    }
}
