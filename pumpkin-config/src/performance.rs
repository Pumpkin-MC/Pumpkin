//! Optional performance backends (allocator / zlib).
//!
//! These settings are **preferences** recorded in `pumpkin.toml`. The actual
//! backend is selected at **compile time** via Cargo features:
//!
//! - `mimalloc` → global allocator = mimalloc
//! - `zlib-ng`  → flate2 uses zlib-ng (else pure-Rust `miniz_oxide`)
//!
//! Default config matches Pumpkin's production performance defaults.

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
    /// - `rust` — pure-Rust `miniz_oxide` (`flate2` `rust_backend`, default)
    /// - `zlib_ng` — zlib-ng (`flate2` `zlib-ng`); requires
    ///   `cargo build --features zlib-ng`
    pub compression_backend: CompressionBackend,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        // Matches pumpkin crate default features: mimalloc + zlib-ng.
        // Override in pumpkin.toml or rebuild with --no-default-features.
        Self {
            allocator: AllocatorBackend::Mimalloc,
            compression_backend: CompressionBackend::ZlibNg,
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
    /// PNX-style best-available pick: whatever backend this binary was built
    /// with (zlib-ng in release builds, miniz_oxide otherwise) without a
    /// mismatch warning.
    #[default]
    Auto,
    /// Pure-Rust `miniz_oxide` (`flate2` feature `rust_backend`).
    ///
    /// `zlib_rs` is accepted as an alias: earlier builds shipped configs with
    /// that spelling, and an unknown variant aborts startup.
    #[serde(alias = "zlib_rs")]
    Rust,
    /// zlib-ng (`flate2` feature `zlib-ng`); high performance, requires a C toolchain.
    ZlibNg,
}

impl CompressionBackend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Rust => "rust",
            Self::ZlibNg => "zlib_ng",
        }
    }
}
