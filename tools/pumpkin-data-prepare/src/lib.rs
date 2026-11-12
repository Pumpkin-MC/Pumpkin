use std::{
    fs::{self, File},
    io::{self, Cursor, Read},
    path::Path,
    time::{Duration, Instant},
};

use sha1::{Digest, Sha1};
use tracing::info;

const MC_VERSION: &str = include_str!("../../../assets/minecraft-version.txt");
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
const PROGRESS_INTERVAL: Duration = Duration::from_secs(2);

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Prepares the datapack for this Minecraft version, reusing an existing extraction.
///
/// Reads `PUMPKIN_MINECRAFT_SERVER_JAR` when set, otherwise downloads the server jar.
/// Returns an error if downloading, validating or extracting the data fails.
pub fn prepare(assets: &Path) -> Result<()> {
    let local_jar = std::env::var_os("PUMPKIN_MINECRAFT_SERVER_JAR");
    prepare_from(
        assets,
        MC_VERSION.trim(),
        local_jar.as_deref().map(Path::new),
    )
}

fn prepare_from(assets: &Path, version: &str, local_jar: Option<&Path>) -> Result<()> {
    let datapack = assets.join("datapack");
    let marker = datapack.join(".minecraft-version");
    if !datapack.join("data/minecraft").is_dir()
        || !fs::read_to_string(&marker).is_ok_and(|prepared| prepared.trim() == version)
    {
        let bytes = if let Some(path) = local_jar {
            info!(
                "Reading local Minecraft server bundle from {}",
                path.display()
            );
            let file = File::open(path)?;
            let length = file.metadata()?.len();
            read_jar(file, Some(length), "local Minecraft server bundle")?
        } else {
            download_server_jar(version)?
        };

        // Store in a tempfile so we can discard any failed extrations
        let staging = tempfile::tempdir_in(assets)?;
        extract_datapack(&bytes, version, staging.path())?;
        copy_conventional_tags(assets, staging.path())?;
        fs::write(
            staging.path().join(".minecraft-version"),
            format!("{version}\n"),
        )?;
        if datapack.exists() {
            fs::remove_dir_all(&datapack)?;
        }
        fs::rename(staging.path(), &datapack)?;
    } else {
        info!("Reusing vanilla datapack for Minecraft {version}");
        copy_conventional_tags(assets, &datapack)?;
    }
    info!("Vanilla datapack for Minecraft {version} is ready");
    Ok(())
}

fn copy_conventional_tags(assets: &Path, datapack: &Path) -> io::Result<()> {
    let source = assets.join("conventional_tags/data/c");
    if source.is_dir() {
        copy_dir_all(&source, &datapack.join("data/c"))?;
    }
    Ok(())
}

fn copy_dir_all(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if !to.is_file() || fs::read(&from)? != fs::read(&to)? {
            fs::copy(from, to)?;
        }
    }
    Ok(())
}

fn megabytes(bytes: u64) -> f64 {
    bytes as f64 / 1_000_000.0
}

fn read_jar(mut source: impl Read, total: Option<u64>, label: &str) -> Result<Vec<u8>> {
    if total.is_some_and(|length| length > MAX_ARCHIVE_BYTES) {
        return Err("Minecraft server jar is too large".into());
    }
    let mut data = Vec::new();
    let mut buffer = [0; 16 * 1024];
    let mut reported = Instant::now();
    loop {
        let count = source.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if data.len() as u64 + count as u64 > MAX_ARCHIVE_BYTES {
            return Err("Minecraft server jar is too large".into());
        }
        data.extend_from_slice(&buffer[..count]);
        if reported.elapsed() >= PROGRESS_INTERVAL {
            if let Some(total) = total {
                info!(
                    "Read {:.1} of {:.1} MB from {label}",
                    megabytes(data.len() as u64),
                    megabytes(total)
                );
            } else {
                info!("Read {:.1} MB from {label}", megabytes(data.len() as u64));
            }
            reported = Instant::now();
        }
    }
    info!("Read {:.1} MB from {label}", megabytes(data.len() as u64));
    Ok(data)
}

fn download_server_jar(version: &str) -> Result<Vec<u8>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::blocking::Client::builder()
        .user_agent("Pumpkin-Data-Prepare/0.1")
        .build()?;
    info!("Fetching version manifest for Minecraft {version}");
    let manifest = read_metadata(client.get(MANIFEST_URL).send()?.error_for_status()?)?;
    let package_url = manifest["versions"]
        .as_array()
        .and_then(|versions| {
            versions
                .iter()
                .find(|entry| entry["id"].as_str() == Some(version))
        })
        .and_then(|entry| entry["url"].as_str())
        .ok_or_else(|| format!("Minecraft {version} is missing from the version manifest"))?;
    let package = read_metadata(client.get(package_url).send()?.error_for_status()?)?;
    let server = &package["downloads"]["server"];
    let url = server["url"].as_str().ok_or("Missing server jar URL")?;
    let checksum = server["sha1"]
        .as_str()
        .ok_or("Missing server jar checksum")?;
    info!("Downloading Minecraft {version} server bundle");
    let response = client.get(url).send()?.error_for_status()?;
    let length = response.content_length();
    let bytes = read_jar(response, length, "Minecraft server bundle")?;
    if hex::encode(Sha1::digest(&bytes)) != checksum {
        return Err("Minecraft server jar checksum does not match its manifest".into());
    }
    Ok(bytes)
}

fn read_metadata(source: impl Read) -> Result<serde_json::Value> {
    const MAX_METADATA_BYTES: u64 = 16 * 1024 * 1024;
    let mut bytes = Vec::new();
    source
        .take(MAX_METADATA_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_METADATA_BYTES {
        return Err("Minecraft version metadata is too large".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn extract_datapack(bytes: &[u8], version: &str, destination: &Path) -> Result<()> {
    let inner = {
        let mut bundler = zip::ZipArchive::new(Cursor::new(bytes))?;
        let entry =
            bundler.by_name(&format!("META-INF/versions/{version}/server-{version}.jar"))?;
        let length = entry.size();
        read_jar(entry, Some(length), "bundled Minecraft server jar")?
    };
    let mut archive = zip::ZipArchive::new(Cursor::new(inner))?;
    let mut extracted_bytes = 0;
    let mut extracted_files = 0;
    let mut reported = Instant::now();
    let entries = archive.len();
    info!("Extracting vanilla datapack");
    for index in 0..entries {
        let mut file = archive.by_index(index)?;
        if !file.name()?.starts_with("data/") {
            continue;
        }
        if file.size() > MAX_ARCHIVE_BYTES - extracted_bytes {
            return Err("Extracted vanilla datapack is too large".into());
        }
        extracted_bytes += file.size();
        let relative = file
            .enclosed_name()
            .ok_or("Invalid datapack archive path")?;
        let path = destination.join(relative);
        if file.is_dir() {
            fs::create_dir_all(path)?;
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            io::copy(&mut file, &mut File::create(path)?)?;
            extracted_files += 1;
        }
        if reported.elapsed() >= PROGRESS_INTERVAL {
            info!(
                "Extracted {extracted_files} files ({:.1} MB) from vanilla datapack so far",
                megabytes(extracted_bytes)
            );
            reported = Instant::now();
        }
    }
    if !destination.join("data/minecraft").is_dir() {
        return Err("Server jar does not contain a vanilla datapack".into());
    }
    info!(
        "Extracted {extracted_files} files ({:.1} MB) from vanilla datapack",
        megabytes(extracted_bytes)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn archive(entries: &[(&str, &[u8])]) -> Result<Vec<u8>> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer.start_file(name, SimpleFileOptions::default())?;
            writer.write_all(bytes)?;
        }
        Ok(writer.finish()?.into_inner())
    }

    fn bundler(entries: &[(&str, &[u8])], version: &str) -> Result<Vec<u8>> {
        let server = archive(entries)?;
        archive(&[(
            &format!("META-INF/versions/{version}/server-{version}.jar"),
            &server,
        )])
    }

    #[test]
    fn preparation_reuses_files_without_invalidating_cargo_inputs() -> Result<()> {
        let assets = tempfile::tempdir()?;
        let jar = assets.path().join("server.jar");
        fs::write(
            &jar,
            bundler(&[("data/minecraft/recipe/test.json", b"{}")], "test")?,
        )?;
        let tags = assets.path().join("conventional_tags/data/c/tags");
        fs::create_dir_all(&tags)?;
        fs::write(tags.join("test.json"), b"{}")?;
        prepare_from(assets.path(), "test", Some(&jar))?;
        let recipe = assets
            .path()
            .join("datapack/data/minecraft/recipe/test.json");
        let copied_tag = assets.path().join("datapack/data/c/tags/test.json");
        let old_time = std::time::UNIX_EPOCH + Duration::from_secs(1);
        let times = fs::FileTimes::new().set_modified(old_time);
        File::options()
            .write(true)
            .open(&recipe)?
            .set_times(times)?;
        File::options()
            .write(true)
            .open(&copied_tag)?
            .set_times(times)?;
        let before = (
            fs::metadata(&recipe)?.modified()?,
            fs::metadata(&copied_tag)?.modified()?,
        );
        fs::remove_file(&jar)?;
        prepare_from(assets.path(), "test", Some(&jar))?;
        assert_eq!(fs::read(&recipe)?, b"{}");
        assert_eq!(
            before,
            (
                fs::metadata(recipe)?.modified()?,
                fs::metadata(copied_tag)?.modified()?
            )
        );
        Ok(())
    }

    #[test]
    fn failed_version_update_preserves_the_previous_datapack() -> Result<()> {
        let assets = tempfile::tempdir()?;
        let jar = assets.path().join("server.jar");
        fs::write(
            &jar,
            bundler(&[("data/minecraft/recipe/test.json", b"old")], "old")?,
        )?;
        prepare_from(assets.path(), "old", Some(&jar))?;
        fs::write(&jar, bundler(&[("not-a-datapack.txt", b"bad")], "new")?)?;
        assert!(prepare_from(assets.path(), "new", Some(&jar)).is_err());
        assert_eq!(
            fs::read_to_string(assets.path().join("datapack/.minecraft-version"))?,
            "old\n"
        );
        assert_eq!(
            fs::read(
                assets
                    .path()
                    .join("datapack/data/minecraft/recipe/test.json")
            )?,
            b"old"
        );
        Ok(())
    }

    #[test]
    fn extraction_rejects_paths_outside_the_datapack() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let bytes = bundler(&[("data/../../outside.txt", b"bad")], "test")?;
        assert!(extract_datapack(&bytes, "test", &directory.path().join("datapack")).is_err());
        assert!(!directory.path().join("outside.txt").exists());
        Ok(())
    }
}
