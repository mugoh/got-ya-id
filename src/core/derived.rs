//! Holds derived attributes
use pyo3::{prelude::PyResult, Python};

#[macro_export]
/// Creates a hashmap from vector key => value pairs
macro_rules! hashmap {
    ($($key: expr => $val: expr), *) =>{{
    let mut map = std::collections::HashMap::new();
    $(map.insert($key, $val);)*
        map
}}}

/// Creates a Python module object
///
/// Python is used to call the Cloudinary API
/// for local-file upload
///
/// Sounds lame :(, but  a direct call to the cloudinary API
/// can't be used in upload of local files
///
/// # Arguments
/// file: &str
///     - The url path of the file to upload
pub fn create_py_mod(file_path: String) -> PyResult<()> {
    // use pyo3::prelude::*;

    let py = Python::acquire_gil();
    upload_static(py.python(), &file_path)
}

/// Perfoms the foreign call to the script that should
/// execute the upload
///
/// # Arguments
/// file: &str
///     - The url path of the file to upload
///
fn upload_static<'a>(py: Python, file_: &'a str) -> PyResult<()> {
    use pyo3::prelude::PyModule;
    use std::fs;

    let script = fs::read_to_string("scr/core/upload")?;

    let loaded_mod = PyModule::from_code(py, &script, "upload", "upload")?;
    let res = loaded_mod.call("upload", (file_,), None)?;
    println!("{:?}", res);
    Ok(())
}
