# pumpkin-codegen

The root [AGENTS.md](../../AGENTS.md) applies here too. This tool turns the extracted vanilla data in `assets/` into the Rust code in `crates/pumpkin-data/src/generated/`.

- Each generator in `src/` is usually named after its input: `block.rs` reads `assets/blocks.json`, `item.rs` reads `items.json`, `loot_table.rs` reads the loot tables in `assets/datapack/`, and so on. `main.rs` lists them all.
- Regenerate everything with `cargo run --locked -p pumpkin-codegen`, or a single output with `-- <output name>`. `-- wit` regenerates the WIT data and the host packet mappings for the plugin API.
- A codegen run can rewrite generated files you didn't mean to change. Keep only the files your change needs and revert the rest.
- If generated data looks wrong at runtime, check the generator before blaming the JSON. A JSON variant the generator doesn't handle can fall through to a default without any error.
- If the value you need isn't in `assets/` at all, it belongs in the [Extractor](https://github.com/Pumpkin-MC/Extractor), not in a hand-written table here.
