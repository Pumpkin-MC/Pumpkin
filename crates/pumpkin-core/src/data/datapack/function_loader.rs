use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_embedded_test_functions<S: std::hash::BuildHasher>(
    functions: &mut HashMap<String, Vec<String>, S>,
) -> usize {
    let before = functions.len();
    for &id in pumpkin_data::test_function::all_test_function_names() {
        if let Some(content) = pumpkin_data::test_function::get_test_function_source(id) {
            functions.insert(id.to_owned(), parse_function_content(content));
        }
    }
    functions.len() - before
}

fn parse_function_content(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.strip_prefix('/').unwrap_or(line).to_string())
        .collect()
}

pub fn load_functions_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    function_dir: &Path,
    functions: &mut HashMap<String, Vec<String>, S>,
) {
    if !function_dir.is_dir() {
        return;
    }
    load_functions_recursive(namespace, function_dir, function_dir, functions);
}

fn load_functions_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    functions: &mut HashMap<String, Vec<String>, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_functions_recursive(namespace, base_dir, &path, functions);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mcfunction"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(s) = stem_path.strip_suffix(".mcfunction") {
                stem_path = s.to_string();
            }
            // Convert Windows backslashes to forward slashes if any
            let stem_path = stem_path.replace('\\', "/");
            let function_id = format!("{namespace}:{stem_path}");
            if let Ok(content) = fs::read_to_string(&path) {
                functions.insert(function_id, parse_function_content(&content));
            }
        }
    }
}

pub fn load_function_tags_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    tags_dir: &Path,
    function_tags: &mut HashMap<String, Vec<String>, S>,
) {
    let function_tags_dir = tags_dir.join("function");
    let function_tags_dir_plural = tags_dir.join("functions");
    for dir in [&function_tags_dir, &function_tags_dir_plural] {
        if dir.is_dir() {
            load_tags_recursive(namespace, dir, dir, function_tags);
        }
    }
}

fn load_tags_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    tags: &mut HashMap<String, Vec<String>, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_tags_recursive(namespace, base_dir, &path, tags);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(s) = stem_path.strip_suffix(".json") {
                stem_path = s.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let tag_id = format!("{namespace}:{stem_path}");
            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(val) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(values_arr) = val.get("values").and_then(serde_json::Value::as_array)
            {
                let list: Vec<String> = values_arr
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect();
                tags.entry(tag_id).or_default().extend(list);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_mcfunction_content() {
        let content = "# This is a comment\nsay Hello world\n/give @p diamond 1\n\n# Another comment\ngive @p stick 5\n";
        let lines = super::parse_function_content(content);

        assert_eq!(
            lines,
            vec![
                "say Hello world".to_string(),
                "give @p diamond 1".to_string(),
                "give @p stick 5".to_string()
            ]
        );
    }
}
