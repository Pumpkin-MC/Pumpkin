use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            load_functions_recursive(namespace, base_dir, &path, functions);
        } else if file_type.is_file()
            && path
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
                let lines: Vec<String> = content
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|line| line.strip_prefix('/').unwrap_or(line).to_string())
                    .collect();
                functions.insert(function_id, lines);
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
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            load_tags_recursive(namespace, base_dir, &path, tags);
        } else if file_type.is_file()
            && path
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
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use super::*;

    fn try_create_dir_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, link)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "symlinks not supported",
            ))
        }
    }

    fn try_create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(target, link)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "symlinks not supported",
            ))
        }
    }

    #[test]
    fn parse_mcfunction_content() {
        let content = "# This is a comment\nsay Hello world\n/give @p diamond 1\n\n# Another comment\ngive @p stick 5\n";
        let lines: Vec<String> = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.strip_prefix('/').unwrap_or(line).to_string())
            .collect();

        assert_eq!(
            lines,
            vec![
                "say Hello world".to_string(),
                "give @p diamond 1".to_string(),
                "give @p stick 5".to_string()
            ]
        );
    }

    #[test]
    fn load_functions_ignores_symlinks_and_loads_regular_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let function_dir = root.join("functions");
        let nested_dir = function_dir.join("sub");
        fs::create_dir_all(&nested_dir).expect("create nested");

        fs::write(nested_dir.join("valid.mcfunction"), "say valid").expect("write valid");

        // External directory with an external function
        let external_dir = root.join("external");
        fs::create_dir_all(&external_dir).expect("create external");
        fs::write(external_dir.join("external.mcfunction"), "say external")
            .expect("write external");

        // Symlink directory inside functions pointing to external directory or parent
        let symlink_dir = function_dir.join("sym_dir");
        let sym_dir_created = try_create_dir_symlink(&external_dir, &symlink_dir).is_ok();

        // Symlink file inside functions pointing to external file
        let symlink_file = function_dir.join("sym_file.mcfunction");
        let sym_file_created =
            try_create_file_symlink(&external_dir.join("external.mcfunction"), &symlink_file)
                .is_ok();

        let mut functions = HashMap::new();
        load_functions_from_dir("test", &function_dir, &mut functions);

        assert!(functions.contains_key("test:sub/valid"));
        assert_eq!(
            functions.get("test:sub/valid").unwrap(),
            &["say valid".to_string()]
        );

        if sym_dir_created {
            assert!(!functions.contains_key("test:sym_dir/external"));
        }
        if sym_file_created {
            assert!(!functions.contains_key("test:sym_file"));
        }
    }

    #[test]
    fn load_function_tags_ignores_symlink_directories() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let tags_dir = root.join("tags");
        let func_tags_dir = tags_dir.join("function");
        fs::create_dir_all(&func_tags_dir).expect("create func tags dir");

        fs::write(
            func_tags_dir.join("valid.json"),
            r#"{"values": ["test:sub/valid"]}"#,
        )
        .expect("write tag json");

        let external_dir = root.join("external_tags");
        fs::create_dir_all(&external_dir).expect("create external tags");
        fs::write(
            external_dir.join("sym_tag.json"),
            r#"{"values": ["test:external"]}"#,
        )
        .expect("write external tag json");

        let symlink_dir = func_tags_dir.join("sym_dir");
        let sym_created = try_create_dir_symlink(&external_dir, &symlink_dir).is_ok();

        let mut tags = HashMap::new();
        load_function_tags_from_dir("test", &tags_dir, &mut tags);

        assert!(tags.contains_key("test:valid"));
        if sym_created {
            assert!(!tags.contains_key("test:sym_dir/sym_tag"));
        }
    }
}
