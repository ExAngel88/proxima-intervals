use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_native")]
fn intervals_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}