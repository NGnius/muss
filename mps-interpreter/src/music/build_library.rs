use std::path::Path;

use super::MpsLibrary;

pub fn build_library<P: AsRef<Path>>(path: P) -> std::io::Result<MpsLibrary> {
    let mut result = MpsLibrary::new();
    result.read_path(path, 10)?;
    Ok(result)
}
