use std::path::PathBuf;

use clap::Parser;

/// Selects the server startup path.
#[derive(Debug, Eq, PartialEq)]
pub enum StartupCommand {
    DedicatedServer,
    GameTestServer(GameTestOptions),
}

/// Holds the options for a `GameTest` server run.
#[derive(Debug, Eq, PartialEq)]
pub struct GameTestOptions {
    pub universe: PathBuf,
    pub report: Option<PathBuf>,
    pub tests: Option<String>,
    pub verify: bool,
    pub repeat_count: i32,
    pub packs: Option<PathBuf>,
}

#[derive(Parser)]
#[command(name = "pumpkin")]
struct GameTestArgs {
    /// The path to where the test server world will be created. Any existing folder will be replaced.
    #[arg(long, default_value = "gametestserver")]
    universe: PathBuf,
    /// Exports results in a junit-like XML report at the given path.
    #[arg(long)]
    report: Option<PathBuf>,
    /// Which test(s) to run (namespaced ID selector using wildcards). Empty means run all.
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    tests: Option<String>,
    /// Runs the tests specified with `test` or `testNamespace` 100 times for each 90 degree rotation step.
    #[arg(long)]
    verify: bool,
    /// Runs each of the specified tests this many times.
    #[arg(
        long = "repeatCount",
        default_value_t = 1,
        value_parser = clap::value_parser!(i32).range(1..)
    )]
    repeat_count: i32,
    /// A folder of datapacks to include in the world.
    #[arg(long)]
    packs: Option<PathBuf>,
}

#[derive(Parser)]
#[command(name = "pumpkin")]
struct DedicatedArgs {
    #[arg(long = "convert-world")]
    _convert_world: Option<String>,
    #[arg(long = "init-settings", alias = "initSettings")]
    _init_settings: bool,
}

/// Selects `GameTest` mode when `--tests` is present and parses its options.
pub fn parse_startup_command(
    args: impl IntoIterator<Item = String>,
) -> Result<StartupCommand, clap::Error> {
    let args: Vec<String> = args.into_iter().collect();
    if !args.iter().any(|arg| {
        arg == "--tests" || arg.starts_with("--tests=") || matches!(arg.as_str(), "-h" | "--help")
    }) {
        DedicatedArgs::try_parse_from(std::iter::once("pumpkin".to_string()).chain(args))?;
        return Ok(StartupCommand::DedicatedServer);
    }

    let options = GameTestArgs::try_parse_from(std::iter::once("pumpkin".to_string()).chain(args))?;
    Ok(StartupCommand::GameTestServer(GameTestOptions {
        universe: options.universe,
        report: options.report,
        tests: options.tests,
        verify: options.verify,
        repeat_count: options.repeat_count,
        packs: options.packs,
    }))
}
