//! Supports Rust - Python calls

use pyo3::{prelude::PyResult, Python};
use serde::{Deserialize, Serialize};

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
pub fn create_py_mod<'a>(file_path: String) -> Result<String, ()> {
    // use pyo3::prelude::*;

    let py = Python::acquire_gil();
    let py = py.python();

    upload_static(py, &file_path).map_err(|e| {
        e.print_and_set_sys_last_vars(py);
    })
}

/// Perfoms the foreign call to the script that should
/// execute the upload
///
/// # Arguments
/// file: &str
///     - The url path of the file to upload
///
fn upload_static<'a>(py: Python, file_: &'a str) -> PyResult<String> {
    use pyo3::prelude::PyModule;
    use std::fs;

    let script = fs::read_to_string("src/core/upload")?;

    let loaded_mod = PyModule::from_code(py, &script, "upload", "upload")?;
    Ok(loaded_mod
        .call("upload", (file_,), None)?
        .as_ref()
        .to_string())
}

/// Holds the key, value response from the sent upload request
#[derive(Serialize, Debug, Deserialize)]
// Desrialize PyDict into struct
pub struct UploadResponse {
    public_id: String,
    //created_at: Option<String>, // Serialize/de chrono datetime
    secure_url: String,
    format: Option<String>,
    url: Option<String>,
}
