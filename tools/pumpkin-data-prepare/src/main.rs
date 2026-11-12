use std::{io::IsTerminal, path::Path, process::ExitCode};

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_ansi(std::io::stdout().is_terminal())
        .init();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    match pumpkin_data_prepare::prepare(&assets) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("Failed to prepare vanilla datapack: {error}");
            ExitCode::FAILURE
        }
    }
}
