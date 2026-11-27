use std::path::PathBuf;

use clap::Parser;

/// Selects the server startup path.
#[derive(Debug, Eq, PartialEq)]
pub enum StartupCommand {
    DedicatedServer(DedicatedServerOptions),
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
    /// The path to the test server world. Only empty or marked `GameTest` folders are replaced.
    #[arg(long, default_value = "gametestserver")]
    universe: PathBuf,
    /// Exports results in a JUnit-like XML report (default: report.xml).
    #[arg(long, num_args = 0..=1, default_missing_value = "report.xml")]
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

/// Holds the recognised options for dedicated startup.
#[derive(Debug, Eq, Parser, PartialEq)]
#[command(name = "pumpkin")]
pub struct DedicatedServerOptions {
    #[arg(long = "convert-world")]
    pub convert_world: Option<String>,
    #[arg(long = "init-settings", alias = "initSettings")]
    pub init_settings: bool,
}

fn parse_dedicated_args(
    args: Vec<String>,
    mut warn_unknown: impl FnMut(&str),
) -> Result<DedicatedServerOptions, clap::Error> {
    let mut known = vec!["pumpkin".to_string()];
    let mut args = args.into_iter().peekable();
    while let Some(arg) = args.next() {
        let (flag, inline_value) = arg
            .split_once('=')
            .map_or((arg.as_str(), false), |(flag, _)| (flag, true));
        match flag {
            "--convert-world" => {
                known.push(arg);
                if !inline_value {
                    known.extend(args.next_if(|value| !value.starts_with('-')));
                }
            }
            "--init-settings" | "--initSettings" => known.push(arg),
            _ => warn_unknown(&arg),
        }
    }
    DedicatedServerOptions::try_parse_from(known)
}

/// Selects `GameTest` mode when `--tests` is present and parses its options.
/// Unknown dedicated server arguments are reported through `warn_unknown`.
pub fn parse_startup_command(
    args: impl IntoIterator<Item = String>,
    warn_unknown: impl FnMut(&str),
) -> Result<StartupCommand, clap::Error> {
    let args: Vec<String> = args.into_iter().collect();
    if !args.iter().any(|arg| {
        arg == "--tests" || arg.starts_with("--tests=") || matches!(arg.as_str(), "-h" | "--help")
    }) {
        return parse_dedicated_args(args, warn_unknown).map(StartupCommand::DedicatedServer);
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{GameTestOptions, StartupCommand, parse_startup_command};

    #[test]
    fn dedicated_startup_ignores_unknown_arguments() {
        let args = [
            "nogui",
            "--convert-world",
            "anvil",
            "--unknown-option",
            "--initSettings",
        ];
        let mut reported = Vec::new();
        let command = parse_startup_command(args.into_iter().map(str::to_owned), |arg| {
            reported.push(arg.to_owned());
        });
        assert!(matches!(
            command,
            Ok(StartupCommand::DedicatedServer(options))
                if options.convert_world.as_deref() == Some("anvil")
                    && options.init_settings
        ));
        assert_eq!(reported, ["nogui", "--unknown-option"]);
    }

    #[test]
    fn startup_still_rejects_invalid_known_options() {
        for args in [
            ["nogui", "--convert-world"],
            ["--tests", "--unknown-option"],
        ] {
            assert!(parse_startup_command(args.into_iter().map(str::to_owned), |_| {}).is_err());
        }
    }

    #[test]
    fn report_flag_uses_default_or_explicit_path() {
        for (args, expected) in [
            (vec!["--tests", "--report"], "report.xml"),
            (vec!["--tests", "--report", "custom.xml"], "custom.xml"),
        ] {
            let command = parse_startup_command(args.into_iter().map(str::to_owned), |_| {});
            assert!(matches!(
                command,
                Ok(StartupCommand::GameTestServer(GameTestOptions {
                    report: Some(path),
                    ..
                })) if path == Path::new(expected)
            ));
        }
    }
}
