use std::collections::HashMap;
use std::path::Path;

use super::file_loader::{collect_files, load_files_parallel};

pub fn load_test_functions<S: std::hash::BuildHasher>(
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
    let files = collect_files(function_dir, "mcfunction");
    let loaded = load_files_parallel(
        namespace,
        function_dir,
        &files,
        "mcfunction",
        |_id, _path, content| Some(parse_function_content(content)),
    );

    for (entry_id, _rel, lines) in loaded {
        functions.insert(entry_id, lines);
    }
}

pub fn load_function_tags_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    tags_dir: &Path,
    function_tags: &mut HashMap<String, Vec<String>, S>,
) {
    for dir in [tags_dir.join("function"), tags_dir.join("functions")] {
        if !dir.is_dir() {
            continue;
        }
        let files = collect_files(&dir, "json");
        let loaded = load_files_parallel(namespace, &dir, &files, "json", |_id, _path, content| {
            let val = serde_json::from_str::<serde_json::Value>(content).ok()?;
            let values_arr = val.get("values")?.as_array()?;
            let list: Vec<String> = values_arr
                .iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect();
            Some(list)
        });

        for (entry_id, _rel, list) in loaded {
            function_tags.entry(entry_id).or_default().extend(list);
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
