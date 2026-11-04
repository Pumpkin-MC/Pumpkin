use std::{
    fmt::Write as _,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use pumpkin::GameTestOptions;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_core::{
    SHOULD_STOP,
    data::VanillaData,
    server::{
        Server,
        server_test_manager::{
            GameTestBatchReport, GameTestQueueEntry, GameTestRetryOptions,
            prepare_test_run_with_reporter, resource_selector_matches,
        },
    },
};
use pumpkin_data::dimension::Dimension;
use pumpkin_gametest::{GameTestReporter, GameTestRunner};
use pumpkin_util::{GameMode, text::TextComponent, world_seed::Seed};
use tracing::info;
use zip::ZipArchive;

const WORLD_NAME: &str = "gametestworld";
const TEST_BASE_X: i32 = 64;
const TEST_BASE_Y: i32 = 180;
const TEST_BASE_Z: i32 = 64;
const TEST_GRID_SPACING: i32 = 128;
const TESTS_PER_ROW: i32 = 8;
const MAX_PACK_ENTRIES: usize = 10_000;
const MAX_PACK_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Default)]
struct LoggingReporter {
    messages: Mutex<Vec<String>>,
}

impl LoggingReporter {
    fn messages(&self) -> Vec<String> {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl GameTestReporter for LoggingReporter {
    fn send_message(&self, message: TextComponent) {
        let text = message.get_text();
        info!(target: "pumpkin::gametest", "{text}");
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(text);
    }
}

struct SilentReporter;

impl GameTestReporter for SilentReporter {
    fn send_message(&self, _message: TextComponent) {}
}

struct TestOutcome {
    name: String,
    required: bool,
    report: Arc<GameTestBatchReport>,
    elapsed_ms: u128,
    messages: Vec<String>,
}

pub async fn run(options: GameTestOptions) -> Result<(), String> {
    let mut advanced_config = AdvancedConfiguration::default();
    advanced_config.networking.java.enabled = false;
    advanced_config.networking.bedrock.enabled = false;
    advanced_config.networking.management.enabled = false;
    advanced_config.plugins.enabled = false;
    pumpkin_core::init_logger(&advanced_config);

    let world_dir = reset_universe(&options.universe)?;
    let packs = copy_packs(options.packs.as_deref(), &world_dir)?;

    let basic_config = BasicConfiguration {
        default_level_name: world_dir.to_string_lossy().into_owned(),
        seed: Seed(0),
        default_gamemode: GameMode::Creative,
        ..Default::default()
    };
    let server = Server::new(
        basic_config,
        advanced_config,
        TelemetryConfig::default(),
        VanillaData::load(),
        Vec::new(),
    )
    .await
    .map_err(|error| format!("Failed to start GameTest server: {error}"))?;

    if !packs.is_empty() {
        server.level_info.rcu(|info| {
            let mut updated = (**info).clone();
            updated.data_packs.enabled.extend(packs.iter().cloned());
            Arc::new(updated)
        });
        server.reload_datapacks(&server);
    }

    let result = run_tests(&server, &options).await;
    server.shutdown().await;
    result
}

async fn run_tests(server: &Arc<Server>, options: &GameTestOptions) -> Result<(), String> {
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let names: Vec<_> = server
        .datapack_manager
        .get_test_instance_names()
        .into_iter()
        .filter(|name| {
            options
                .tests
                .as_deref()
                .is_none_or(|selector| resource_selector_matches(selector, name))
                && server
                    .datapack_manager
                    .get_test_instance(name)
                    .is_some_and(|test| !test.manual_only)
        })
        .collect();
    if names.is_empty() {
        return Err("No GameTests matched the selection".to_string());
    }

    info!(target: "pumpkin::gametest", count = names.len(), "Starting GameTests");
    let mut outcomes = Vec::new();
    for (index, test_id) in names.iter().enumerate() {
        let test = server
            .datapack_manager
            .get_test_instance(test_id)
            .ok_or_else(|| format!("GameTest '{test_id}' disappeared during selection"))?;
        let rotations = if options.verify { 4 } else { 1 };
        let runs = if options.verify {
            100
        } else {
            options.repeat_count.max(1)
        };
        let index = i32::try_from(index).map_err(|_| "Too many GameTests".to_string())?;
        let test_x = TEST_BASE_X + index.rem_euclid(TESTS_PER_ROW) * TEST_GRID_SPACING;
        let test_z = TEST_BASE_Z + index.div_euclid(TESTS_PER_ROW) * TEST_GRID_SPACING;

        for rotation in 0..rotations {
            let sink = Arc::new(LoggingReporter::default());
            let report = Arc::new(GameTestBatchReport::new(Arc::new(SilentReporter), 1));
            let entry = GameTestQueueEntry::new(
                test_id,
                world.clone(),
                test_x,
                test_z,
                rotation,
                GameTestRetryOptions::new(runs, false),
                report.clone(),
            )
            .with_y(TEST_BASE_Y);
            let manager = prepare_test_run_with_reporter(server, entry, sink.clone())
                .await
                .map_err(|error| format!("Failed to prepare GameTest '{test_id}': {error}"))?;
            let mut runner = GameTestRunner::new();
            runner.enqueue(manager);
            let started = Instant::now();
            while !runner.is_empty() {
                if SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err("GameTest server interrupted".to_string());
                }
                server.tick();
                runner.tick().await;
            }
            outcomes.push(TestOutcome {
                name: if options.verify {
                    format!("{test_id} (rotation {rotation})")
                } else {
                    test_id.clone()
                },
                required: test.required,
                report,
                elapsed_ms: started.elapsed().as_millis(),
                messages: sink.messages(),
            });
        }
    }

    if let Some(path) = &options.report {
        write_report(path, &outcomes)?;
    }
    let required_failures: usize = outcomes
        .iter()
        .map(|outcome| outcome.report.failed_required())
        .sum();
    let optional_failures: usize = outcomes
        .iter()
        .map(|outcome| outcome.report.failed_optional())
        .sum();
    let runs: usize = outcomes
        .iter()
        .map(|outcome| outcome.report.total_runs())
        .sum();
    info!(target: "pumpkin::gametest", runs, required_failures, optional_failures, "GameTests complete");
    if required_failures == 0 {
        Ok(())
    } else {
        Err(format!(
            "{required_failures} required GameTest run(s) failed"
        ))
    }
}

fn reset_universe(universe: &Path) -> Result<PathBuf, String> {
    if universe.exists() {
        let canonical = universe.canonicalize().map_err(|error| error.to_string())?;
        let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
        if cwd.starts_with(&canonical) {
            return Err(format!(
                "Refusing to replace the current directory or its parent: {}",
                universe.display()
            ));
        }
        fs::remove_dir_all(universe)
            .map_err(|error| format!("Failed to reset GameTest universe: {error}"))?;
    }
    let world_dir = universe.join(WORLD_NAME);
    fs::create_dir_all(&world_dir)
        .map_err(|error| format!("Failed to create GameTest universe: {error}"))?;
    Ok(world_dir)
}

fn copy_packs(source: Option<&Path>, world_dir: &Path) -> Result<Vec<String>, String> {
    let Some(source) = source else {
        return Ok(Vec::new());
    };
    if !source.exists() {
        return Ok(Vec::new());
    }
    let target = world_dir.join("datapacks");
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    let mut enabled = Vec::new();
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() && path.join("pack.mcmeta").is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            copy_directory(&path, &target.join(&name)).map_err(|error| error.to_string())?;
            enabled.push(format!("file/{name}"));
        } else if path.extension().is_some_and(|extension| extension == "zip") {
            let name = path
                .file_stem()
                .ok_or_else(|| format!("Invalid pack name: {}", path.display()))?
                .to_string_lossy()
                .into_owned();
            let destination = target.join(&name);
            extract_zip(&path, &destination).map_err(|error| error.to_string())?;
            if destination.join("pack.mcmeta").is_file() {
                enabled.push(format!("file/{name}"));
            }
        }
    }
    Ok(enabled)
}

fn copy_directory(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let destination = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_directory(&path, &destination)?;
        } else if file_type.is_file() {
            fs::copy(path, destination)?;
        }
    }
    Ok(())
}

fn extract_zip(source: &Path, target: &Path) -> io::Result<()> {
    let file = File::open(source)?;
    let mut archive = ZipArchive::new(file)?;
    if archive.len() > MAX_PACK_ENTRIES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Zip pack has too many entries",
        ));
    }
    let mut remaining_bytes = MAX_PACK_BYTES;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        if entry.size() > remaining_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Zip pack is too large",
            ));
        }
        remaining_bytes -= entry.size();
        let relative = entry.enclosed_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Zip pack contains an invalid path",
            )
        })?;
        let destination = target.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut output = File::create(destination)?;
            io::copy(&mut entry.take(MAX_PACK_BYTES + 1), &mut output)?;
        }
    }
    Ok(())
}

fn write_report(path: &Path, outcomes: &[TestOutcome]) -> Result<(), String> {
    let failures = outcomes
        .iter()
        .filter(|outcome| outcome.report.failed_required() + outcome.report.failed_optional() > 0)
        .count();
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuite name=\"GameTests\" tests=\"{}\" failures=\"{failures}\">\n",
        outcomes.len()
    );
    for outcome in outcomes {
        let _ = write!(
            xml,
            "  <testcase name=\"{}\" time=\"{:.3}\" attempts=\"{}\" required=\"{}\">",
            escape_xml(&outcome.name),
            outcome.elapsed_ms as f64 / 1000.0,
            outcome.report.total_runs(),
            outcome.required
        );
        if outcome.report.failed_required() + outcome.report.failed_optional() > 0 {
            let failure = outcome
                .messages
                .iter()
                .find(|message| message.contains(" failed!"))
                .map_or("GameTest failed", String::as_str);
            let _ = write!(xml, "<failure message=\"{}\" />", escape_xml(failure));
        }
        xml.push_str("</testcase>\n");
    }
    xml.push_str("</testsuite>\n");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, xml).map_err(|error| format!("Failed to write GameTest report: {error}"))
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
