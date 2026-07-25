# AGENTS.md — Pumpkin Project Guidelines

## Project Overview
Pumpkin is a lightweight, high-performance Minecraft server implementation written in Rust. It aims to achieve high concurrency, low latency, and modular protocol handling using async I/O.

## Tech Stack
- **Language:** Rust (Edition 2021+)
- **Async Runtime:** Tokio
- **Networking & Framing:** `tokio-util`, `bytes`
- **Serialization:** `serde`, custom Minecraft VarInt/VarLong codecs
- **Logging/Tracing:** `tracing`

---

## Build, Test, and Quality Commands

Always run these commands to verify code changes before finalizing tasks:

- **Check compilation:** `cargo check --all-targets`
- **Run test suite:** `cargo test`
- **Lint code:** `cargo clippy -- -D warnings`
- **Format code:** `cargo fmt --check`
- **Run single test:** `cargo test <test_name>`

---

## Repository Structure

- `crates/` or `src/`
  - `net/` — Networking stack, TCP listeners, packet codecs, connection state machine.
  - `protocol/` — Minecraft packet definitions (Handshake, Status, Login, Play).
  - `world/` — Chunk management, world generation, block state handling.
  - `entity/` — Player and entity representations, spatial indexing.
  - `server/` — Server tick loop, command processing, player session management.

---

## Core Technical Rules for Agents

1. **Async & Tokio Rules:**
   - Never block the Tokio runtime thread. Use `tokio::task::spawn_blocking` for heavy filesystem or cryptographic operations.
   - Prefer passing owned `Bytes` or `BytesMut` slices rather than cloning large heap allocated buffers.

2. **Error Handling:**
   - Use strongly typed domain errors with `thiserror` in library crates.
   - Never use `.unwrap()` or `.expect()` in production code paths or packet handlers. Always propagate errors via `Result`.

3. **Concurrency & Mutexes:**
   - Avoid holding locks across `.await` points to prevent deadlocks.
   - Prefer std `Mutex` or `rwlock` for fast in-memory operations if held briefly; use `tokio::sync::Mutex` only when locks must cross `.await` boundaries.

4. **Minecraft Protocol Compliance:**
   - Follow official Minecraft protocol specifications for packet structure and VarInt serialization.
   - Maintain strict separation between packet parsing (raw bytes -> Rust struct) and packet handling (state transitions / world mutations).

---

## Agent Task Execution Checklist

When modifying or adding code:
1. Locate relevant structs/modules before editing.
2. Ensure new network packets implement the standard read/write trait.
3. Add unit tests for serialization, deserialization, and edge cases (e.g., malformed VarInts).
4. Run `cargo check` and `cargo test` to verify build integrity.
