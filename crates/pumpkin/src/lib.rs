use std::path::PathBuf;

use clap::Parser;

/// Selects the server startup path.
#[derive(Debug, Eq, PartialEq)]
pub enum StartupCommand {
    DedicatedServer,
    GameTestServer(GameTestOptions),
}

/// Holds the options for a GameTest server run.
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
    #[arg(long, action = clap::ArgAction::Set, default_value_t = false)]
    verify: bool,
    /// Runs each of the specified tests this many times.
    #[arg(long = "repeatCount", default_value_t = 1)]
    repeat_count: i32,
    /// A folder of datapacks to include in the world.
    #[arg(long)]
    packs: Option<PathBuf>,
}

/// Selects GameTest mode when `--tests` is present and parses its options.
pub fn parse_startup_command(
    args: impl IntoIterator<Item = String>,
) -> Result<StartupCommand, clap::Error> {
    let args: Vec<String> = args.into_iter().collect();
    if !args.iter().any(|arg| {
        arg == "--tests" || arg.starts_with("--tests=") || matches!(arg.as_str(), "-h" | "--help")
    }) {
        return Ok(StartupCommand::DedicatedServer);
    }

    let options = GameTestArgs::try_parse_from(std::iter::once("pumpkin".to_string()).chain(args))?;
    Ok(StartupCommand::GameTestServer(GameTestOptions {
        universe: options.universe,
        report: options.report,
        tests: options.tests.filter(|selector| !selector.is_empty()),
        verify: options.verify,
        repeat_count: options.repeat_count,
        packs: options.packs,
    }))
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::{GameTestOptions, StartupCommand, parse_startup_command};

    #[test]
    fn dedicated_is_the_default() {
        assert_eq!(
            parse_startup_command([]).ok(),
            Some(StartupCommand::DedicatedServer)
        );
        assert_eq!(
            parse_startup_command(["--convert-world".into(), "anvil".into()]).ok(),
            Some(StartupCommand::DedicatedServer)
        );
    }

    #[test]
    fn parses_game_test_options() {
        let args = [
            "--universe=tmp/tests",
            "--report",
            "results.xml",
            "--tests",
            "minecraft:*",
            "--verify",
            "true",
            "--repeatCount",
            "3",
            "--packs",
            "packs",
        ]
        .map(str::to_owned);
        let parsed = parse_startup_command(args);
        assert_eq!(
            parsed.ok(),
            Some(StartupCommand::GameTestServer(GameTestOptions {
                universe: "tmp/tests".into(),
                report: Some("results.xml".into()),
                tests: Some("minecraft:*".into()),
                verify: true,
                repeat_count: 3,
                packs: Some("packs".into()),
            }))
        );
    }

    #[test]
    fn tests_without_selector_runs_all() {
        let parsed = parse_startup_command(["--tests".into(), "--verify".into(), "true".into()]);
        assert!(matches!(
            parsed,
            Ok(StartupCommand::GameTestServer(GameTestOptions {
                tests: None,
                verify: true,
                ..
            }))
        ));
    }

    #[test]
    fn help_and_invalid_values_are_handled_by_clap() {
        assert_eq!(
            parse_startup_command(["--help".into()])
                .err()
                .map(|error| error.kind()),
            Some(ErrorKind::DisplayHelp)
        );
        assert!(parse_startup_command(["--tests".into(), "--verify".into()]).is_err());
        assert!(
            parse_startup_command(["--tests".into(), "--repeatCount".into(), "nope".into()])
                .is_err()
        );
    }
}
