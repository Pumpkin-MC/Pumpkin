use anyhow::{Context, Result, bail};
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [plugin, helpers, output] = args.as_slice() else {
        bail!("Usage: pumpkin-plugin-compose <plugin.wasm> <helpers.wasm> <output.wasm>");
    };
    let plugin = fs::read(plugin).context("failed to read plugin component")?;
    let helpers = fs::read(helpers).context("failed to read helper component")?;
    let bytes = pumpkin_plugin_compose::compose(&plugin, &helpers)?;
    fs::write(output, bytes).context("failed to write composed plugin")
}
