//! Resolution of configuration values that are stored outside the config file.
//!
//! Secrets such as the Velocity forwarding secret are awkward to keep in
//! `pumpkin.toml`: the file is commonly committed to version control, baked into
//! an image, or shared when asking for support. Letting a value point at a file
//! instead means the secret can live somewhere with tighter permissions, or be
//! supplied by Docker/Kubernetes secrets, which mount exactly this way.

use std::fs;

/// Resolves a configuration value that may reference a file.
///
/// A value beginning with `@` is treated as the path to a file whose contents
/// are the real value, e.g. `secret = "@forwarding.secret"`. Relative paths are
/// resolved against the server's working directory, which is where
/// `pumpkin.toml` itself is read from. Any other value is used literally.
///
/// A leading `@` can be escaped by doubling it, so `@@value` resolves to the
/// literal `@value`. Without this a secret that genuinely starts with `@` would
/// be impossible to express.
///
/// Surrounding whitespace is trimmed from the file's contents, because nearly
/// everything that writes a secret file leaves a trailing newline behind.
///
/// `what` names the value being resolved and is used only in panic messages.
///
/// Panics if the referenced file cannot be read, or if it resolves to an empty
/// value. Both mean the server would otherwise start with a silently wrong
/// secret and reject every connection, which is far harder to diagnose.
pub fn resolve_file_reference(value: &str, what: &str) -> String {
    value.strip_prefix('@').map_or_else(
        || value.to_owned(),
        |rest| {
            rest.strip_prefix('@').map_or_else(
                || read_referenced_file(rest, what),
                |literal| format!("@{literal}"),
            )
        },
    )
}

/// Reads `what` from the file at `path`, trimming surrounding whitespace.
fn read_referenced_file(path: &str, what: &str) -> String {
    assert!(
        !path.is_empty(),
        "{what} is `@` with no file path after it. Use `@<path>` to read the value from a file, or `@@` to start a literal value with `@`"
    );

    let contents = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Couldn't read {what} from file `{path}`. Reason: {err}"));

    let trimmed = contents.trim();
    assert!(
        !trimmed.is_empty(),
        "{what} was read from file `{path}`, but that file is empty"
    );

    trimmed.to_owned()
}

#[cfg(test)]
mod tests {
    use super::resolve_file_reference;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Writes `contents` to a temporary file and returns it alongside the `@`
    /// reference that points at it.
    fn file_containing(contents: &str) -> (NamedTempFile, String) {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write_all(contents.as_bytes())
            .expect("failed to write temp file");
        let reference = format!("@{}", file.path().display());
        (file, reference)
    }

    #[test]
    fn plain_value_is_used_literally() {
        assert_eq!(resolve_file_reference("hunter2", "test value"), "hunter2");
    }

    #[test]
    fn empty_plain_value_is_left_alone() {
        // An unset secret is only an error when the feature using it is enabled,
        // so resolution itself must not reject it.
        assert_eq!(resolve_file_reference("", "test value"), "");
    }

    #[test]
    fn reference_is_read_from_the_file() {
        let (_file, reference) = file_containing("s3cret-from-disk");
        assert_eq!(
            resolve_file_reference(&reference, "test value"),
            "s3cret-from-disk"
        );
    }

    #[test]
    fn surrounding_whitespace_is_trimmed() {
        // `echo secret > file` and Docker secrets both leave a trailing newline.
        let (_file, reference) = file_containing("  s3cret\n");
        assert_eq!(resolve_file_reference(&reference, "test value"), "s3cret");
    }

    #[test]
    fn doubled_at_escapes_to_a_literal_value() {
        assert_eq!(
            resolve_file_reference("@@not-a-path", "test value"),
            "@not-a-path"
        );
    }

    #[test]
    #[should_panic(expected = "Couldn't read test value from file")]
    fn missing_file_panics() {
        resolve_file_reference("@definitely/not/a/real/path.secret", "test value");
    }

    #[test]
    #[should_panic(expected = "that file is empty")]
    fn empty_file_panics() {
        let (_file, reference) = file_containing("\n  \n");
        resolve_file_reference(&reference, "test value");
    }

    #[test]
    #[should_panic(expected = "no file path after it")]
    fn bare_at_panics() {
        resolve_file_reference("@", "test value");
    }
}
