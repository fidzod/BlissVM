use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum PreprocessorError {
    FileNotFound(String),
    CircularInclude(String),
}

pub fn expand(
    source: &str,
    base_dir: &Path,
    already_seen: Option<HashSet<PathBuf>>,
) -> Result<String, PreprocessorError> {
    let mut output = Vec::new();
    let mut already_seen = already_seen.unwrap_or_default();

    for line in source.lines() {
        let Some(filename) = line.strip_prefix(".include ") else {
            output.push(line.to_string());
            continue;
        };

        let filename = filename.trim_matches('"');

        let file_path = base_dir.join(filename);

        if !already_seen.insert(file_path.to_path_buf()) {
            return Err(PreprocessorError::CircularInclude(
                file_path.to_string_lossy().to_string(),
            ));
        }

        let contents = fs::read_to_string(file_path.clone())
            .map_err(|_| PreprocessorError::FileNotFound(filename.to_string()))?;

        output.extend(
            expand(
                &contents,
                file_path.parent().unwrap_or(base_dir),
                Some(already_seen.clone()),
            )?
            .lines()
            .map(String::from),
        );
    }

    Ok(output.join("\n"))
}
