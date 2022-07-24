//! Supports Rust - Python calls

use pyo3::{prelude::PyResult, Python};
use serde::{Deserialize, Serialize};

use std::{borrow::Cow, collections::HashMap};

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
pub fn create_py_mod(file_path: String, dir_path: &'_ str) -> Result<String, ()> {
    // use pyo3::prelude::*;

    let py = Python::acquire_gil();
    let py = py.python();

    upload_static(py, &file_path, dir_path).map_err(|e| {
        e.print_and_set_sys_last_vars(py);
    })
}

///  Deletes a static cloudinary file matiching
///  the given public ID
pub fn remove_py_mod(file_path: &str) -> Result<String, ()> {
    let py = Python::acquire_gil();
    let py = py.python();

    delete_static(py, &file_path).map_err(|e| {
        e.print_and_set_sys_last_vars(py);
    })
}

/// Makes a call to the script executing the delete
fn delete_static<'a>(py: Python, file_id: &'a str) -> PyResult<String> {
    //
    use pyo3::prelude::{Py, PyAny, PyModule};
    use std::fs;

    let script = fs::read_to_string("src/core/upload")?;

    let loaded_mod: Py<PyAny> = PyModule::from_code(py, &script, "", "")?
        .getattr("destroy")?
        .into();

    Ok(loaded_mod.call(py, (file_id,), None)?.to_string())
}

/// Perfoms the foreign call to the script that should
/// execute the upload
///
/// # Arguments
/// file: &str
///     - The url path of the file to upload
///
fn upload_static<'a>(py: Python, file_: &'a str, dir_: &'a str) -> PyResult<String> {
    use pyo3::prelude::{Py, PyAny, PyModule};
    use std::fs;

    let script = fs::read_to_string("src/core/upload")?;

    let loaded_mod: Py<PyAny> = PyModule::from_code(py, &script, "", "")?
        .getattr("upload")?
        .into();
    let res = loaded_mod.call(py, (file_, dir_), None)?.to_string();

    let res = res.replace("{", "").replace("'", "\"").replace("}", "");
    let res = res.split(',').collect::<Vec<&str>>();
    let mut res_map = std::collections::HashMap::new();

    for item in &res {
        let key_value = item.splitn(2, ':').collect::<Vec<&str>>();
        res_map.insert(
            key_value[0]
                .trim_matches(|x: char| x.is_whitespace())
                .replace("\"", ""),
            key_value[1]
                .trim_matches(|x: char| x.is_whitespace())
                .replace("\"", ""),
        );
    }

    Ok(res_map.get("secure_url").unwrap().to_owned())
}

/// Holds the key, value response from the sent upload request
#[derive(Serialize, Deserialize)]
// Deserialize PyDict into struct
pub struct UploadResponse<'a> {
    public_id: Cow<'a, str>,
    created_at: Cow<'a, str>,
    secure_url: Cow<'a, str>,
    format: Cow<'a, str>,
    url: Cow<'a, str>,
    tags: Cow<'a, str>,
}

impl<'a> UploadResponse<'a> {
    /// Creates a new upload response instance
    ///
    /// # Arguments
    /// ## map_data: _Hashmap<str, str>_
    /// - Key, value pairs from which to extract the field and
    /// equivalent values
    pub fn new<'b>(map_data: HashMap<&'b str, &'b str>) -> UploadResponse<'b> {
        UploadResponse {
            public_id: Cow::Borrowed(map_data.get("public_id").unwrap()),
            created_at: Cow::Borrowed(map_data.get("created_at").unwrap()),
            secure_url: Cow::Borrowed(map_data.get("secure_url").unwrap()),
            format: Cow::Borrowed(map_data.get("format").unwrap()),
            url: Cow::Borrowed(map_data.get("url").unwrap()),
            tags: Cow::Borrowed(map_data.get("tags").unwrap()),
        }
    }
}
