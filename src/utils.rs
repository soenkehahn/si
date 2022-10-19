use std::path::Path;

pub fn render_path<P: AsRef<Path>>(path: P) -> String {
    let result = path.as_ref().to_string_lossy().into_owned();
    if result.starts_with(|c| c == '/' || c == '.') {
        result
    } else {
        format!("./{}", result)
    }
}
