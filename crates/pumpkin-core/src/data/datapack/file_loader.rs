//! Shared helpers for reading datapack directories, optionally in parallel.

use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

/// Recursively collects files under `dir` whose extension matches `extension`
/// (ASCII case-insensitive), sorted so loading order is deterministic.
#[must_use]
pub fn collect_files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_into(dir, extension, &mut files);
    files.sort_unstable();
    files
}

fn collect_files_into(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(
                "Failed to read datapack directory {}: {error}",
                dir.display()
            );
            return;
        }
    };
    for entry in entries.filter_map(|entry| {
        entry
            .inspect_err(|error| {
                tracing::warn!(
                    "Failed to read directory entry in {}: {error}",
                    dir.display()
                );
            })
            .ok()
    }) {
        let path = entry.path();
        let Ok(kind) = entry.file_type().inspect_err(|error| {
            tracing::warn!("Failed to read file type for {}: {error}", path.display());
        }) else {
            continue;
        };
        if kind.is_dir() {
            collect_files_into(&path, extension, files);
        } else if (kind.is_file() || (kind.is_symlink() && path.is_file()))
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            files.push(path);
        }
    }
}

/// Resource path of `path` relative to `base_dir`, with the `.{extension}`
/// suffix removed and backslashes normalised to `/`.
fn resource_path(base_dir: &Path, path: &Path, extension: &str) -> Option<String> {
    let rel_path = path.strip_prefix(base_dir).ok()?;
    if rel_path.extension().is_some_and(|ext| ext == extension) {
        resource_path_without_extension(base_dir, path)
    } else {
        Some(rel_path.to_string_lossy().replace('\\', "/"))
    }
}

/// Like [`resource_path`], but strips whatever extension the file has, matching
/// `Path::file_stem` semantics used by the structure loaders.
#[must_use]
pub fn resource_path_without_extension(base_dir: &Path, path: &Path) -> Option<String> {
    let rel_path = path.strip_prefix(base_dir).ok()?;
    Some(
        rel_path
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

/// Reads and parses each file in `files` in parallel.
///
/// Returns `(entry_id, resource_path, value)` in the sorted order of `files`.
/// Files that fail to read, or whose `parse` returns `None`, are skipped.
#[must_use]
pub fn load_files_parallel<T, F>(
    namespace: &str,
    base_dir: &Path,
    files: &[PathBuf],
    extension: &str,
    parse: F,
) -> Vec<(String, String, T)>
where
    T: Send,
    F: Fn(&str, &Path, &str) -> Option<T> + Sync,
{
    files
        .par_iter()
        .filter_map(|path| {
            let rel_path = resource_path(base_dir, path, extension)?;
            let entry_id = format!("{namespace}:{rel_path}");
            let content = fs::read_to_string(path)
                .inspect_err(|error| {
                    tracing::warn!("Failed to read datapack file {}: {error}", path.display());
                })
                .ok()?;
            let value = parse(&entry_id, path, &content)?;
            Some((entry_id, rel_path, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_insensitive_discovery_with_matching_suffix_produces_the_same_resource_path() {
        for (name, extension) in [("reward.json", "json"), ("reward.JSON", "JSON")] {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(name), "{}").unwrap();
            let files = collect_files(dir.path(), "json");
            let loaded =
                load_files_parallel("test", dir.path(), &files, extension, |_, _, _| Some(()));
            assert_eq!(loaded, [("test:reward".into(), "reward".into(), ())]);
        }
    }

    #[cfg(unix)]
    #[test]
    fn discovery_skips_directory_symlinks_and_special_files() {
        use std::os::unix::{fs::symlink, net::UnixListener};

        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let file = nested.join("reward.json");
        fs::write(&file, "{}").unwrap();
        symlink(dir.path(), nested.join("cycle")).unwrap();
        symlink(&nested, dir.path().join("alias")).unwrap();
        let file_link = dir.path().join("linked.JSON");
        symlink(&file, &file_link).unwrap();
        let _socket = UnixListener::bind(dir.path().join("socket.json")).unwrap();
        assert_eq!(collect_files(dir.path(), "json"), [file_link, file]);
    }
}
