use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum PreprocessorError {
    FileNotFound(String),
    CircularInclude(String),
    MalformedDefinition
}

pub fn expand(source: &str, base_dir: &Path) -> Result<String, PreprocessorError> {
    let mut seen = HashSet::new();
    let mut defines = HashMap::new();
    expand_inner(source, base_dir, &mut seen, &mut defines)
}

fn substitute(line: &str, defines: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut token = String::new();

    for c in line.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            token.push(c);
        } else {
            if !token.is_empty() {
                result.push_str(defines.get(&token).map(String::as_str).unwrap_or(&token));
                token.clear();
            }
            result.push(c);
        }
    }

    if !token.is_empty() {
        result.push_str(defines.get(&token).map(String::as_str).unwrap_or(&token));
    }
    result
}

fn expand_inner(
    source: &str,
    base_dir: &Path,
    already_seen: &mut HashSet<PathBuf>,
    defines: &mut HashMap<String, String>,
) -> Result<String, PreprocessorError> {
    let mut output = Vec::new();

    for line in source.lines() {
        if let Some(filename) = line.strip_prefix(".include ") {
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
                expand_inner(
                    &contents,
                    file_path.parent().unwrap_or(base_dir),
                    already_seen,
                    defines,
                )?
                .lines()
                .map(String::from),
            );
        } else if let Some(definition) = line.strip_prefix(".define ") {
            let Some((name, value)) = definition.split_once(' ') else {
                return Err(PreprocessorError::MalformedDefinition)
            };
            defines.insert(name.to_string(), value.to_string());
        } else {
            output.push(substitute(line, defines));
        }
    }

    Ok(output.join("\n"))
}

