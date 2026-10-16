//! Heap memory and tick CPU profiling on Windows, macOS and Linux with glibc.
#![cfg(all(
    any(
        target_os = "windows",
        target_os = "macos",
        all(target_os = "linux", target_env = "gnu")
    ),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod allocator;
mod capture;
mod collector;
pub mod cpu;
mod platform;
mod profile;
mod sampling;
mod session;

pub use allocator::{TrackingAllocator, register_allocator, without_tracking};
pub use session::{Operation, PROFILER, Profiler, Request};
