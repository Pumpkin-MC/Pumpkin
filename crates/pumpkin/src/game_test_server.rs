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
            GameTestQueueEntry, game_test_grid_spacing, prepare_test_run_with_reporter,
        },
    },
};
use pumpkin_data::dimension::Dimension;
use pumpkin_gametest::{
    GameTestBatchReport, GameTestReporter, GameTestRetryOptions, GameTestRotation, GameTestRunner,
    VERIFY_TEST_BATCH_SIZE, resource_selector_matches,
};
use pumpkin_util::{GameMode, text::TextComponent, world_seed::Seed};
use tracing::info;
use zip::ZipArchive;

const WORLD_NAME: &str = "gametestworld";
const TEST_BASE_X: i32 = 64;
const TEST_BASE_Y: i32 = 180;
const TEST_BASE_Z: i32 = 64;
const TESTS_PER_ROW: i32 = 8;
const MAX_TESTS_PER_BATCH: usize = 50;
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

struct TestGrid {
    next_x: i32,
    next_z: i32,
    row_depth: i32,
    column: i32,
}

impl TestGrid {
    const fn new() -> Self {
        Self {
            next_x: TEST_BASE_X,
            next_z: TEST_BASE_Z,
            row_depth: 0,
            column: 0,
        }
    }

    fn place(&mut self, width: i32, depth: i32) -> Result<(i32, i32), String> {
        let position = (self.next_x, self.next_z);
        self.next_x = self.next_x.checked_add(width).ok_or("Too many GameTests")?;
        self.row_depth = self.row_depth.max(depth);
        self.column += 1;
        if self.column == TESTS_PER_ROW {
            self.column = 0;
            self.next_x = TEST_BASE_X;
            self.next_z = self
                .next_z
                .checked_add(self.row_depth)
                .ok_or("Too many GameTests")?;
            self.row_depth = 0;
        }
        Ok(position)
    }
}

pub async fn run(options: GameTestOptions) -> Result<(), String> {
    let mut advanced_config = AdvancedConfiguration::default();
    advanced_config.networking.java.enabled = false;
    advanced_config.networking.bedrock.enabled = false;
    advanced_config.networking.management.enabled = false;
    advanced_config.plugins.enabled = false;
    pumpkin_core::init_logger(&advanced_config);

    let pack_source = validate_pack_source(options.packs.as_deref(), &options.universe)?;
    let world_dir = reset_universe(&options.universe)?;
    let packs = copy_packs(pack_source.as_deref(), &world_dir)?;

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
    let names: Vec<_> = server
        .datapack_manager
        .get_test_instance_names()
        .into_iter()
        .filter(|name| {
            options
                .tests
                .as_deref()
                .is_none_or(|selector| selector.is_empty() || resource_selector_matches(selector, name))
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
    let verify = options.tests.is_some() && options.verify;
    let copies = if verify {
        VERIFY_TEST_BATCH_SIZE
    } else if options.tests.is_some() {
        usize::try_from(options.repeat_count).map_err(|error| error.to_string())?
    } else {
        1
    };
    let rotations = if verify {
        &GameTestRotation::ALL[..]
    } else {
        &GameTestRotation::ALL[..1]
    };
    let mut grid = TestGrid::new();
    for test_id in &names {
        for &rotation in rotations {
            outcomes.push(
                run_test_rotation(server, test_id, rotation, copies, verify, &mut grid).await?,
            );
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

async fn run_test_rotation(
    server: &Arc<Server>,
    test_id: &str,
    rotation: GameTestRotation,
    copies: usize,
    verify: bool,
    grid: &mut TestGrid,
) -> Result<TestOutcome, String> {
    let test = server
        .datapack_manager
        .get_test_instance(test_id)
        .ok_or_else(|| format!("GameTest '{test_id}' disappeared during selection"))?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let sink = Arc::new(LoggingReporter::default());
    let report = Arc::new(GameTestBatchReport::new(Arc::new(SilentReporter), copies));
    let mut runner = GameTestRunner::new();
    let (column_spacing, row_spacing) = game_test_grid_spacing(server, test_id, rotation)
        .await
        .map_err(|error| format!("Failed to prepare GameTest '{test_id}': {error}"))?;
    let started = Instant::now();

    for _ in 0..copies {
        let (test_x, test_z) = grid.place(column_spacing, row_spacing)?;

        let entry = GameTestQueueEntry::new(
            test_id,
            world.clone(),
            test_x,
            test_z,
            rotation.steps(),
            GameTestRetryOptions::no_retries(),
            report.clone(),
        )
        .with_y(TEST_BASE_Y);
        let manager = prepare_test_run_with_reporter(server, entry, sink.clone())
            .await
            .map_err(|error| format!("Failed to prepare GameTest '{test_id}': {error}"))?;
        runner.enqueue(manager);
        if runner.active_len() == MAX_TESTS_PER_BATCH {
            run_batch(server, &mut runner).await?;
        }
    }
    run_batch(server, &mut runner).await?;

    Ok(TestOutcome {
        name: if verify {
            format!("{test_id} (rotation {})", rotation.steps())
        } else {
            test_id.to_string()
        },
        required: test.required,
        report,
        elapsed_ms: started.elapsed().as_millis(),
        messages: sink.messages(),
    })
}

async fn run_batch(server: &Arc<Server>, runner: &mut GameTestRunner) -> Result<(), String> {
    while !runner.is_empty() {
        if SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("GameTest server interrupted".to_string());
        }
        server.tick();
        runner.tick().await;
    }
    Ok(())
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

fn validate_pack_source(source: Option<&Path>, universe: &Path) -> Result<Option<PathBuf>, String> {
    let Some(source) = source else {
        return Ok(None);
    };
    let source = source.canonicalize().map_err(|error| {
        format!(
            "Failed to access GameTest packs at {}: {error}",
            source.display()
        )
    })?;
    if !source.is_dir() {
        return Err(format!(
            "GameTest packs source is not a directory: {}",
            source.display()
        ));
    }
    if universe.exists() {
        let universe = universe
            .canonicalize()
            .map_err(|error| format!("Failed to access GameTest universe: {error}"))?;
        if source.starts_with(&universe) {
            return Err(format!(
                "GameTest packs source is inside the universe being replaced: {}",
                source.display()
            ));
        }
    }
    Ok(Some(source))
}

fn copy_packs(source: Option<&Path>, world_dir: &Path) -> Result<Vec<String>, String> {
    let Some(source) = source else {
        return Ok(Vec::new());
    };
    if !source.is_dir() {
        return Err(format!(
            "GameTest packs source is not a directory: {}",
            source.display()
        ));
    }
    let target = world_dir.join("datapacks");
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    let mut enabled = Vec::new();
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() && path.join("pack.mcmeta").is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let destination = target.join(&name);
            if destination.exists() {
                return Err(format!(
                    "Duplicate GameTest pack destination: {}",
                    destination.display()
                ));
            }
            copy_directory(&path, &destination).map_err(|error| error.to_string())?;
            enabled.push(format!("file/{name}"));
        } else if path.extension().is_some_and(|extension| extension == "zip") {
            let name = path
                .file_stem()
                .ok_or_else(|| format!("Invalid pack name: {}", path.display()))?
                .to_string_lossy()
                .into_owned();
            let destination = target.join(&name);
            if destination.exists() {
                return Err(format!(
                    "Duplicate GameTest pack destination: {}",
                    destination.display()
                ));
            }
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
        .chars()
        .filter(|&character| {
            matches!(
                character,
                '\u{9}'
                    | '\u{A}'
                    | '\u{D}'
                    | '\u{20}'..='\u{D7FF}'
                    | '\u{E000}'..='\u{FFFD}'
                    | '\u{10000}'..='\u{10FFFF}'
            )
        })
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::escape_xml;

    #[test]
    fn xml_escaping_removes_forbidden_controls() {
        assert_eq!(
            escape_xml("test\u{1}\u{1b}\u{fffe}&<>\"'\t\n\r"),
            "test&amp;&lt;&gt;&quot;&apos;\t\n\r"
        );
    }
}
