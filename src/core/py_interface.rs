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
pub fn create_py_mod(file_path: String) -> Result<String, ()> {
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
    let res = loaded_mod
        .call("upload", (file_,), None)?
        .as_ref()
        .to_string();

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
#[derive(Serialize, Debug, Deserialize)]
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
