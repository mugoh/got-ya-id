//! Holds derived attributes
use std::error;

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
fn create_py_mod() -> Result<(), Box<dyn error::Error>> {
    use py03::Python;
    let gil = Python::acquire_gil();
    gil.python?
}

/// Perfoms the foreign call to the script that should
/// execute the upload
///
/// # Arguments
/// file: &str
///     - The url path of the file to upload
///
pub fn upload_static<'a>(file: &'a str) -> Result<(), Box<dyn error::Error>> {
    use py03::PyModule;
    use std::fs;

    let script = fs::read_to_string("scr/core/upload")?;
    let loaded_mod = PyModule::from_code(create_py_mod(), script, "upload", "upload")?;
    loaded_mod.call("upload", (file,), None).extract()?
}
