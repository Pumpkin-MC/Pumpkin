# CLAUDE.md — Pumpkin Codebase Guide

This document describes the Pumpkin project for AI assistants working on the codebase.

## Project Overview

Pumpkin is a Minecraft server written entirely in Rust, targeting **Minecraft 1.21.11** (protocol version 772+). It supports both **Java Edition** (TCP) and **Bedrock Edition** (UDP/RakNet) clients. The project prioritizes performance, vanilla game mechanics compatibility, and extensibility through a plugin system.

- **License:** GPLv3
- **Rust Edition:** 2024
- **Minimum Rust Version:** 1.89 (stable toolchain)
- **Docs:** https://pumpkinmc.org/
- **Discord:** https://discord.gg/wT8XjrjKkf

## Quick Reference — Commands

```bash
# Build
cargo build
cargo build --release          # LTO + strip + 1 codegen unit

# Test
cargo nextest run --verbose    # Primary test runner
cargo test --doc --verbose     # Doctests (separate step)

# Lint
cargo fmt --check              # Format check
cargo clippy --all-targets --all-features  # Debug lint
cargo clippy --release --all-targets --all-features  # Release lint

# Benchmarks (Criterion)
cargo bench -p pumpkin-world
cargo bench -p pumpkin-data
```

**CI enforces `RUSTFLAGS="-Dwarnings"` — all warnings are errors.**

## Workspace Structure

The project is a Cargo workspace with 9 crates:

```
Pumpkin/
├── pumpkin/              # Main server binary + library
├── pumpkin-protocol/     # Minecraft protocol (packets, encryption, compression)
├── pumpkin-world/        # World generation, chunks, biomes, lighting
├── pumpkin-util/         # Shared types, traits, math, noise, permissions
├── pumpkin-data/         # Build-time generated game data (blocks, items, etc.)
├── pumpkin-nbt/          # Named Binary Tag format (Anvil region files)
├── pumpkin-inventory/    # Inventory management, crafting, recipes
├── pumpkin-config/       # TOML-based server configuration
├── pumpkin-macros/       # Proc macros for data and packet generation
└── pumpkin-api-macros/   # Proc macros for the plugin event system
```

### Crate Dependency Flow

```
pumpkin (binary)
├── pumpkin-protocol   → pumpkin-util, pumpkin-nbt, pumpkin-data, pumpkin-macros
├── pumpkin-world      → pumpkin-util, pumpkin-nbt, pumpkin-data, pumpkin-macros
├── pumpkin-inventory  → pumpkin-protocol, pumpkin-world, pumpkin-data, pumpkin-util
├── pumpkin-config     → pumpkin-util
└── pumpkin-api-macros
```

`pumpkin-util` is the shared foundation — all other crates depend on it.

## Main Binary Layout (`pumpkin/src/`)

| Directory     | Purpose                                        |
|---------------|------------------------------------------------|
| `main.rs`     | Server bootstrap (tokio async entry point)     |
| `lib.rs`      | `PumpkinServer` struct, TCP/UDP listeners      |
| `server/`     | Tick loop, scheduler, player management        |
| `entity/`     | Players, mobs, AI, physics, collision          |
| `net/`        | Java and Bedrock client connection handling     |
| `world/`      | Runtime world management, chunk loading        |
| `block/`      | Block states, redstone logic, pistons          |
| `command/`    | Command dispatcher, vanilla commands           |
| `item/`       | Item behaviors and interactions                |
| `plugin/`     | Plugin loader, event system, API exports       |
| `data/`       | Runtime data loading (bans, whitelist, ops)    |

## Key Architectural Patterns

### Async Runtime
- **Tokio** multi-threaded runtime (`#[tokio::main]`)
- CPU-intensive work (world gen, chunk loading) uses **Rayon** thread pools
- Never block Tokio on Rayon — use `tokio::sync::mpsc` to bridge runtimes
- `CancellationToken` and `TaskTracker` for graceful shutdown

### Networking
- Java: TCP with AES-128-CFB8 encryption + zlib compression
- Bedrock: UDP via RakNet protocol
- Connection states: Handshake → Status → Login → Config → Play
- Packet codecs in `pumpkin-protocol/src/codec/`

### Data Generation
- `pumpkin-data` generates 37+ Rust source files at build time from JSON registries
- Build script: `pumpkin-data/build/build.rs`
- Generated output: `pumpkin-data/src/generated/`
- Uses Rayon for parallel generation, runs `rustfmt` on output
- Do not hand-edit files in `src/generated/`

### Plugin System
- Dynamic loading via `libloading` (.so/.dll)
- Plugins implement the `Plugin` trait
- 50+ event types across server/world/player/block/entity domains
- Async and blocking event handlers with priority ordering
- Cancellable events for player actions, block changes, etc.

### Configuration
- Two TOML config files at runtime:
  - `configuration.toml` — basic settings (port, difficulty, motd, max players)
  - `features.toml` — advanced (logging, networking, world, chat, PvP)
- Auto-merges new fields with defaults on load

## Code Quality Rules

### Clippy (Strict)
All four lint groups are set to **deny**: `all`, `nursery`, `pedantic`, `cargo`.

Key enforced rules:
- No `println!`/`eprintln!` — use `log` crate macros instead (`print_stdout`, `print_stderr` denied)
- No `dbg!` macro
- Use `Self` in impl blocks (`use_self`)
- No redundant clones or collects
- No empty enum variant/struct brackets
- Use `if_then_some_else_none` and `option_if_let_else` patterns

Allowed exceptions include: cast precision/sign loss, floating-point comparison, missing panic/error docs, module name repetitions.

### Formatting
- `rustfmt.toml`: edition 2024
- 4-space indentation, LF line endings, UTF-8
- Max line length: 100 characters (`.editorconfig`)

### Naming Conventions
- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Block/item IDs: `u16` type aliases (`BlockId`, `BlockStateId`)

## Testing

- **Primary runner:** `cargo nextest run` (faster than `cargo test`)
- **Doc tests:** `cargo test --doc` (run separately)
- **Benchmarks:** Criterion-based, in `benches/` directories
- **Plugin tests:** `cargo test -p pumpkin --lib plugin`
- **Config test feature:** `pumpkin-config` has a `test_helper` feature for runtime config changes in tests

## CI/CD (GitHub Actions)

The `rust.yml` workflow runs on every push and PR:

1. `cargo fmt --check`
2. `cargo-machete` (unused dependency detection)
3. `cargo clippy` (debug + release modes)
4. `cargo nextest run` + `cargo test --doc` on 6 platforms:
   - Ubuntu (x86-64, ARM64)
   - Windows (x86-64, ARM64)
   - macOS (ARM64, x86-64)
5. Release builds with platform-specific optimizations
6. Draft release on master push

Additional workflows: Docker multi-platform build (`docker.yml`), typo checking (`typos.yml`).

## Important Files

| File                    | Purpose                                    |
|-------------------------|--------------------------------------------|
| `Cargo.toml`           | Workspace config, lint rules, profiles     |
| `rust-toolchain.toml`  | Stable channel, rust-analyzer component    |
| `rustfmt.toml`         | Edition 2024 formatting                    |
| `.editorconfig`        | IDE settings (indent, line endings)        |
| `typos.toml`           | Typo checker whitelist                     |
| `Dockerfile`           | Alpine-based multi-stage build             |
| `docker-compose.yml`   | Local Docker testing                       |
| `flake.nix`            | Nix dev environment                        |

## Development Tips

- **Do not use `println!`** — the `print_stdout` and `print_stderr` clippy lints are denied. Use `log::info!`, `log::debug!`, etc.
- **Do not edit generated files** in `pumpkin-data/src/generated/`. Modify the build script or JSON source data instead.
- **Rayon for CPU work, Tokio for I/O** — see `pumpkin_world::level::Level::fetch_chunks` for the canonical pattern of bridging the two.
- **`pumpkin-util` is shared foundation** — changes here affect all crates. Be cautious with modifications.
- **Block state IDs are `u16`** — there are 7000+ block state variants. Use the generated lookup functions in `pumpkin-data`.
- **Protocol versions** — the server targets Minecraft 1.21.11. Protocol packet definitions live in `pumpkin-protocol/src/client/` and `pumpkin-protocol/src/server/`.
- **Feature flags** — `pumpkin-protocol` has `serverbound`/`clientbound`/`query` features. `pumpkin-config` has `test_helper`. `pumpkin-world` has `tokio_taskdump`.

## Agent Orchestration System (`.claude/`)

This repository uses an agent-based development coordination system:

- **`.claude/contracts/`** — Per-agent ownership boundaries (which crates/paths each agent may modify)
- **`.claude/specs/`** — Minecraft 1.21.4 game data, registry definitions, Bukkit API references
- **`.claude/sessions/`** — Session logs and decision records (append-only)
- **`.claude/ORCHESTRATOR.md`** — Constitution defining agent coordination rules
- **`.claude/rules/session-protocol.md`** — Mandatory read-before-write protocol for agents

**Agent roster:** Architect, Core, Protocol, WorldGen, Entity, Items, Redstone, Storage, Plugin — each owns specific crate paths defined in their contract TOML.
