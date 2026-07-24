//! Optional performance backends (allocator / zlib).
//!
//! These settings are **preferences** recorded in `pumpkin.toml`. The actual
//! backend is selected at **compile time** via Cargo features:
//!
//! - `mimalloc` → global allocator = mimalloc
//! - `zlib-rs`  → flate2 uses zlib-rs (else pure-Rust miniz_oxide)
//!
//! Default config keeps vanilla-friendly defaults: system malloc + rust zlib.

use serde::{Deserialize, Serialize};

/// Performance-related optional backends.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PerformanceConfig {
    /// Global memory allocator preference.
    ///
    /// - `system` — OS default allocator (always used unless built with `mimalloc`)
    /// - `mimalloc` — requires `cargo build --features mimalloc`
    pub allocator: AllocatorBackend,

    /// DEFLATE / zlib implementation used by flate2 (network + some storage paths).
    ///
    /// - `rust` — pure-Rust miniz_oxide (`flate2` `rust_backend`, default)
    /// - `zlib_rs` — pure-Rust zlib-rs (`flate2` `zlib-rs`); requires
    ///   `cargo build --features zlib-rs`
    pub compression_backend: CompressionBackend,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        // Matches pumpkin crate default features: mimalloc + zlib-rs.
        // Override in pumpkin.toml or rebuild with --no-default-features.
        Self {
            allocator: AllocatorBackend::Mimalloc,
            compression_backend: CompressionBackend::ZlibRs,
        }
    }
}

/// Global allocator selection (compile-time feature must match for full effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AllocatorBackend {
    /// OS / libc allocator (glibc, Windows heap, …).
    #[default]
    System,
    /// Microsoft mimalloc — multi-threaded friendly.
    Mimalloc,
}

impl AllocatorBackend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Mimalloc => "mimalloc",
        }
    }
}

/// flate2 / compression backend preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompressionBackend {
    /// Pure-Rust miniz_oxide (`flate2` feature `rust_backend`).
    #[default]
    Rust,
    /// Pure-Rust zlib-rs (`flate2` feature `zlib-rs`); usually faster than miniz_oxide,
    /// no C toolchain required (unlike zlib-ng).
    ZlibRs,
}

impl CompressionBackend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::ZlibRs => "zlib_rs",
        }
    }
}
