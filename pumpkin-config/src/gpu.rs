//! GPU compute acceleration configuration.
//!
//! Controls whether and how the server uses GPU-accelerated compute shaders for
//! terrain noise generation and light propagation.  All fields are optional in the
//! TOML file — missing keys fall back to the [`Default`] values below.

use serde::{Deserialize, Serialize};

/// Top-level GPU acceleration configuration section.
///
/// Placed under `[gpu]` in `pumpkin.toml`.  When `enabled = false` the GPU path is
/// never attempted, regardless of the other settings — the server operates
/// identically to a build without the `gpu` Cargo feature.
///
/// # Example (pumpkin.toml)
///
/// ```toml
/// [gpu]
/// enabled = true
/// noise_acceleration = true
/// light_acceleration = true
/// backend = "auto"
///
/// [gpu.device]
/// strategy = "name"
/// name = "RTX"
/// ```
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct GpuConfig {
    /// Master switch.  When `false` the GPU is never initialised and every
    /// acceleration path falls back to the CPU, identical to a build without
    /// the `gpu` feature.
    ///
    /// Default: `true`
    pub enabled: bool,

    /// How the server picks a physical GPU device.  See [`GpuDeviceSelection`].
    pub device: GpuDeviceSelection,

    /// Accelerate density-function (noise / terrain shape) evaluation on the GPU.
    ///
    /// This covers octave-perlin noise samplers, splines, and the full
    /// density-function graph interpreter.  On supported hardware the
    /// stage-level speedup is 12-110× depending on batch size.
    ///
    /// Default: `true`
    pub noise_acceleration: bool,

    /// Accelerate sky-light and block-light column scanning on the GPU.
    ///
    /// The GPU scan computes light levels for thousands of blocks per dispatch,
    /// amortising the wgpu submission overhead.  Useful on headless servers
    /// where the GPU would otherwise sit idle.
    ///
    /// Default: `true`
    pub light_acceleration: bool,

    /// Force a specific graphics backend rather than letting wgpu auto-detect.
    /// See [`GpuBackend`] for available choices.
    ///
    /// Default: `"auto"`
    pub backend: GpuBackend,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            device: GpuDeviceSelection::default(),
            noise_acceleration: true,
            light_acceleration: true,
            backend: GpuBackend::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Device selection
// ---------------------------------------------------------------------------

/// How the server picks a physical GPU adapter.
///
/// # TOML representation
///
/// The device selection **must** be a table, not a bare string.
///
/// ```toml
/// # Auto (default — the table may be omitted entirely):
/// [gpu.device]
/// strategy = "auto"
///
/// # Pick by index:
/// [gpu.device]
/// strategy = "index"
/// index = 1   # second GPU
///
/// # Pick by name substring (case-insensitive):
/// [gpu.device]
/// strategy = "name"
/// name = "RTX 4090"
///
/// # Prefer integrated GPU (laptops / power-saving):
/// [gpu.device]
/// strategy = "integrated"
/// ```
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(tag = "strategy", rename_all = "lowercase")]
pub enum GpuDeviceSelection {
    /// Automatically pick the best available GPU.
    ///
    /// wgpu sorts adapters by power preference (discrete > integrated > CPU),
    /// so this selects the most capable hardware present in the system.
    ///
    /// Default.
    #[default]
    #[serde(rename = "auto")]
    Auto,

    /// Select by zero-based adapter index.
    ///
    /// `index = 0` is the first adapter wgpu enumerates (typically the
    /// discrete GPU).  Useful on multi-GPU servers where you want a specific
    /// card, e.g. the one not connected to a display.
    #[serde(rename = "index")]
    Index {
        /// Zero-based adapter index.
        index: u32,
    },

    /// Select the first adapter whose name contains the given substring
    /// (case-insensitive match).
    ///
    /// Useful when adapter ordering is unpredictable across reboots or
    /// driver updates: name-based selection is stable.
    #[serde(rename = "name")]
    Name {
        /// Substring to match against `adapter.get_info().name`.
        name: String,
    },

    /// Prefer an integrated GPU.
    ///
    /// Maps to `wgpu::PowerPreference::LowPower`.  Suitable for laptops or
    /// energy-constrained deployments where the discrete GPU should be
    /// reserved for a display.
    #[serde(rename = "integrated")]
    Integrated,
}

// ---------------------------------------------------------------------------
// Backend forcing
// ---------------------------------------------------------------------------

/// Force a specific graphics API backend.
///
/// wgpu picks the best native backend automatically (Vulkan on Linux, Metal on
/// macOS, DX12 on Windows).  Forcing a backend is mainly useful in CI
/// environments, containers that expose a specific API, or when triaging
/// backend-specific driver bugs.
///
/// # TOML
///
/// ```toml
/// [gpu]
/// backend = "vulkan"
/// ```
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GpuBackend {
    /// Let wgpu pick the best available backend for the platform.
    ///
    /// Default.
    #[default]
    #[serde(rename = "auto")]
    Auto,

    /// Force the Vulkan backend (`wgpu::Backend::Vulkan`).
    ///
    /// Available on Linux and Windows (with Vulkan drivers installed).
    #[serde(rename = "vulkan")]
    Vulkan,

    /// Force the Metal backend (`wgpu::Backend::Metal`).
    ///
    /// Available on macOS and (via `MoltenVK`) on other platforms.
    #[serde(rename = "metal")]
    Metal,

    /// Force the DirectX 12 backend (`wgpu::Backend::Dx12`).
    ///
    /// Available on Windows only.  Requires DX12-capable hardware and drivers.
    #[serde(rename = "dx12")]
    Dx12,

    /// Force the OpenGL / GLES backend (`wgpu::Backend::Gl`).
    ///
    /// Broadest compatibility but lowest performance.  Useful as a fallback
    /// when native backends are unavailable (e.g. older hardware, software
    /// rendering via llvmpipe).
    #[serde(rename = "gl")]
    Gl,
}

impl GpuBackend {
    /// Human-readable backend name for log messages.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Vulkan => "vulkan",
            Self::Metal => "metal",
            Self::Dx12 => "dx12",
            Self::Gl => "opengl",
        }
    }
}
