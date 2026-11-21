use std::fs;
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, fchown};
use std::path::Path;

use tempfile::NamedTempFile;
use toml_edit::{DocumentMut, Item, Value};

/// Sets a boolean `key` in the top-level table of the TOML file at `path`. Comments and the
/// other keys and tables are kept as they are.
///
/// A missing file is created with only that key. A file that isn't valid TOML or is read-only,
/// and a symlink whose target is missing, are left untouched.
///
/// # Errors
///
/// Returns an error if the file can't be read, parsed or replaced.
pub fn set_top_level_bool(path: &Path, key: &str, value: bool) -> io::Result<()> {
    // Write through a symlink instead of replacing it.
    let path = match fs::canonicalize(path) {
        Ok(path) => path,
        // Renaming over the link itself would turn it into a regular file.
        Err(err) if path.is_symlink() => {
            return Err(io::Error::new(
                err.kind(),
                format!("couldn't resolve the symlink: {err}"),
            ));
        }
        Err(_) => path.to_path_buf(),
    };
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };
    let mut document: DocumentMut = source
        .parse()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    let mut value = Value::from(value);
    if let Some(Item::Value(old)) = document.get_mut(key) {
        // Keep the spacing and the comment around the old value.
        *value.decor_mut() = old.decor().clone();
        *old = value;
    } else {
        document.insert(key, Item::Value(value));
    }

    write_file_atomically(&path, document.to_string().as_bytes())
}

/// Writes `contents` to a new file next to `path` and renames it over `path`, so a crash leaves
/// either the old or the new file.
fn write_file_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => return Err(err),
    };
    if metadata
        .as_ref()
        .is_some_and(|metadata| metadata.permissions().readonly())
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the file is read-only",
        ));
    }

    let mut file = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;
    // The file can hold secrets, so the new one keeps the group and permissions of the old one.
    if let Some(metadata) = metadata {
        #[cfg(unix)]
        fchown(file.as_file(), None, Some(metadata.gid())).map_err(|err| {
            io::Error::new(err.kind(), format!("couldn't keep the file's group: {err}"))
        })?;
        file.as_file().set_permissions(metadata.permissions())?;
    }
    file.write_all(contents)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|err| err.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::set_top_level_bool;

    #[test]
    fn changes_only_the_value() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pumpkin.toml");
        fs::write(
            &path,
            "# Admin notes\nseed = \"42\"\nwhite_list   =   false # turned on by hand\nenforce_whitelist = false\n\n[networking.java]\n# Proxy port\naddress = \"0.0.0.0:25565\"\n",
        )
        .unwrap();

        set_top_level_bool(&path, "white_list", true).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "# Admin notes\nseed = \"42\"\nwhite_list   =   true # turned on by hand\nenforce_whitelist = false\n\n[networking.java]\n# Proxy port\naddress = \"0.0.0.0:25565\"\n"
        );
    }

    #[test]
    fn adds_a_missing_key_to_the_top_level_table() {
        // Appending the key at the end of the file would put it in `[networking.java]`.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pumpkin.toml");
        fs::write(
            &path,
            "seed = \"42\"\n\n[networking.java]\naddress = \"0.0.0.0:25565\"\n",
        )
        .unwrap();

        set_top_level_bool(&path, "white_list", true).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "seed = \"42\"\nwhite_list = true\n\n[networking.java]\naddress = \"0.0.0.0:25565\"\n"
        );
    }

    #[test]
    fn creates_a_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pumpkin.toml");

        set_top_level_bool(&path, "white_list", true).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "white_list = true\n");
    }

    #[test]
    fn leaves_an_invalid_file_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pumpkin.toml");
        let source = "white_list = false\n[networking.java\naddress = \"0.0.0.0:25565\"\n";
        fs::write(&path, source).unwrap();

        assert!(set_top_level_bool(&path, "white_list", true).is_err());

        assert_eq!(fs::read_to_string(&path).unwrap(), source);
    }

    #[test]
    fn leaves_a_read_only_file_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pumpkin.toml");
        fs::write(&path, "white_list = false\n").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).unwrap();

        assert!(set_top_level_bool(&path, "white_list", true).is_err());

        assert_eq!(fs::read_to_string(&path).unwrap(), "white_list = false\n");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn keeps_the_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pumpkin.toml");
        fs::write(&path, "white_list = false\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        set_top_level_bool(&path, "white_list", true).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o640);
    }

    #[cfg(unix)]
    #[test]
    fn writes_through_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("server.toml");
        let link = dir.path().join("pumpkin.toml");
        fs::write(&target, "white_list = false\n").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        set_top_level_bool(&link, "white_list", true).unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(&target).unwrap(), "white_list = true\n");
    }

    #[cfg(unix)]
    #[test]
    fn leaves_a_broken_symlink_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("server.toml");
        let link = dir.path().join("pumpkin.toml");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(set_top_level_bool(&link, "white_list", true).is_err());

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
