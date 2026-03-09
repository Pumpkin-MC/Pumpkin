use std::{
    backtrace::{Backtrace, BacktraceStatus},
    fmt::{Display, Write as _},
    fs::{File, create_dir_all},
    panic::{Location as PanicLocation, PanicHookInfo},
    path::{Path, PathBuf},
    thread::{self, Thread},
};

use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
use pumpkin_world::CURRENT_MC_VERSION;
use rustc_hash::FxHashMap;
use sysinfo::{Cpu, System};
use time::OffsetDateTime;
use tracing::error;

pub const BYTES_PER_MEBIBYTE: u64 = 1024 * 1024;

/// Writes to a string which cannot fail.
macro_rules! writeln_output {
    ($dst:expr $(,)?) => {
        writeln!($dst).unwrap();
    };
    ($dst:expr, $($arg:tt)*) => {
        writeln!($dst, $($arg)*).unwrap();
    };
}

/// A backtrace that either references
/// a full backtrace already generated,
/// or a new one.
pub enum FullBacktrace {
    Captured,
    ForceCaptured(Backtrace),
}

/// Represents the location of a character
/// in a file.
pub struct Location {
    pub file_name: String,
    pub line: u32,
    pub column: u32,
}

impl From<&PanicLocation<'_>> for Location {
    fn from(value: &PanicLocation) -> Self {
        Self {
            file_name: value.file().to_string(),
            line: value.line(),
            column: value.column(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file_name, self.line, self.column)
    }
}

/// Represents a crash report, containing
/// the required information.
///
/// This is stored without a lifetime related to a [`PanicHookInfo`]
/// so that information from a reference to a [`PanicHookInfo`]
/// can be stored globally without lifetime issues.
pub struct CrashReport {
    utc_time: OffsetDateTime,
    payload: Option<String>,
    thread: Thread,
    panic_location: Option<Location>,
    full_backtrace: FullBacktrace,
    captured_backtrace: Backtrace,
}

impl CrashReport {
    /// Creates a new crash report detailing the state of the
    /// server at the panic handler call site.
    pub fn new(
        panic_info: &PanicHookInfo<'_>,
        captured_backtrace: Backtrace,
        full_backtrace: FullBacktrace,
    ) -> Self {
        Self {
            utc_time: OffsetDateTime::now_utc(),
            payload: {
                let payload = panic_info.payload();
                payload.downcast_ref::<&str>().map_or_else(
                    || payload.downcast_ref::<String>().cloned(),
                    |s| Some(s.to_string()),
                )
            },
            thread: thread::current(),
            full_backtrace,
            captured_backtrace,
            panic_location: panic_info.location().map(Into::into),
        }
    }

    /// Prints information about the crash to the console,
    /// but not the full report.
    #[allow(clippy::print_stderr)]
    pub fn print_to_console(&self) {
        const RED: Color = Color::Named(NamedColor::Red);

        error!(
            "{}",
            TextComponent::text("Pumpkin has encountered a panic!")
                .color(RED)
                .bold()
                .to_pretty_console()
        );

        error!("");

        // Printing panic info.
        let thread_name = self.thread.name().unwrap_or("<unnamed>");
        let thread_id = self.thread.id();

        let message = self.panic_location.as_ref().map_or_else(
            || format!("Thread '{thread_name}' {thread_id:?} panicked"),
            |location| format!("Thread '{thread_name}' with {thread_id:?} panicked at {location}"),
        );

        if let Some(payload) = &self.payload {
            error!("{}:", RED.console_color(&message));
            error!("{payload}");
        } else {
            error!("{}", RED.console_color(&message));
        }

        error!("");

        let backtrace_status = self.full_backtrace().status();

        match backtrace_status {
            BacktraceStatus::Unsupported => {
                error!(
                    "{}",
                    RED.console_color("Backtracing is not supported for this platform.")
                );
            }
            // It cannot possibly be BacktraceStatus::Disabled
            // as it is a forced backtrace.
            BacktraceStatus::Captured => {
                error!(
                    "{}",
                    RED.console_color("The full backtrace will be printed to the crash report.")
                );

                if self.captured_backtrace.status() == BacktraceStatus::Captured {
                    eprintln!(
                        "{}\n{}",
                        RED.console_color("Backtrace:"),
                        &self.captured_backtrace
                    );
                }
            }
            _ => {
                error!("{}", RED.console_color("Backtrace status is unknown, so no backtrace will be generated for the crash report."));
            }
        }
    }

    /// Generates the file content of the crash report file
    /// that would be generated from this report.
    pub fn generate_file_content(&self) -> String {
        let mut output = String::new();

        writeln_output!(&mut output, "====== Pumpkin Crash Report ======");
        writeln_output!(&mut output);
        writeln_output!(&mut output, "Time: {}", self.utc_time);
        writeln_output!(
            &mut output,
            "Message: {}",
            self.payload.as_deref().unwrap_or("<unknown>")
        );
        writeln_output!(&mut output);
        writeln_output!(&mut output, "--- Panicking Thread ---");
        writeln_output!(&mut output, "ID: {:?}", self.thread.id());
        if let Some(thread_name) = self.thread.name() {
            writeln_output!(&mut output, "Name: {}", thread_name);
        }
        writeln_output!(&mut output, "Backtrace:");
        writeln_output!(&mut output, "{}", self.full_backtrace());

        writeln_output!(&mut output, "--- Server Details ---");

        writeln_output!(
            &mut output,
            "Pumpkin Version: {}",
            Self::get_pumpkin_version()
        );
        writeln_output!(&mut output, "Minecraft Version: {}", CURRENT_MC_VERSION);
        writeln_output!(
            &mut output,
            "Server compiled with Rust {}",
            rustc_version_runtime::version()
        );

        if sysinfo::IS_SUPPORTED_SYSTEM {
            writeln_output!(&mut output, "\n--- System Details ---");

            let mut sys = System::new_all();
            sys.refresh_all();

            writeln_output!(
                &mut output,
                "Operating System: {}",
                System::long_os_version().unwrap_or("Unknown".to_string())
            );
            writeln_output!(&mut output, "Kernel: {}", System::kernel_long_version());
            writeln_output!(
                &mut output,
                "Physical Memory: {} MiB/{} MiB used, {} MiB free",
                sys.used_memory() / BYTES_PER_MEBIBYTE,
                sys.total_memory() / BYTES_PER_MEBIBYTE,
                sys.free_memory() / BYTES_PER_MEBIBYTE
            );
            writeln_output!(
                &mut output,
                "Swap Memory: {} MiB/{} MiB used, {} MiB free",
                sys.used_swap() / BYTES_PER_MEBIBYTE,
                sys.total_swap() / BYTES_PER_MEBIBYTE,
                sys.free_swap() / BYTES_PER_MEBIBYTE
            );

            Self::write_cpus(&mut output, &sys);
        }

        output
    }

    fn write_cpus(output: &mut String, sys: &System) {
        writeln_output!(output);
        let cpus = sys.cpus();

        writeln_output!(output, "Total cores: {}", cpus.len());
        writeln_output!(output);

        let mut different_brands: FxHashMap<(&str, &str), Vec<&Cpu>> = FxHashMap::default();

        // `sysinfo` provides us a CPU for each core, so we try to group them.
        for cpu in cpus {
            different_brands
                .entry((cpu.brand(), cpu.vendor_id()))
                .or_default()
                .push(cpu);
        }

        for (i, ((brand, vendor_id), cpus)) in different_brands.iter().enumerate() {
            let prefix = format!(" CPU #{:<5}", i + 1);
            let padded = " ".repeat(prefix.len());

            let names = cpus
                .iter()
                .map(|cpu| cpu.name())
                .collect::<Vec<&str>>()
                .join(", ");

            let avg_freq = cpus.iter().map(|cpu| cpu.frequency()).sum::<u64>() / cpus.len() as u64;

            writeln_output!(output, "|{prefix} Cores: {}", cpus.len());
            writeln_output!(output, "|{padded} Names: {}", names);
            writeln_output!(output, "|{padded} Brand: {}", brand);
            writeln_output!(output, "|{padded} Average Frequency: {} MHz", avg_freq);
            writeln_output!(output, "|{padded} Vendor ID: {}", vendor_id);
            writeln_output!(output);
        }
    }

    #[must_use]
    pub fn get_pumpkin_version() -> String {
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        format!(
            "{} (Commit: {}/{})",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            profile
        )
    }

    /// Saves this report to the `crash-reports` directory.
    ///
    /// Returns a `Result` containing a path if successful.
    pub fn save(&self) -> std::io::Result<PathBuf> {
        const CRASH_REPORTS_DIR: &str = "./crash-reports";

        let file_name = format!(
            "crash-{}-{:02}-{:02}_{:02}.{:02}.{:02}.txt",
            self.utc_time.year(),
            self.utc_time.month() as u8,
            self.utc_time.day(),
            self.utc_time.hour(),
            self.utc_time.minute(),
            self.utc_time.second()
        );

        let path = Path::new(CRASH_REPORTS_DIR).join(file_name);
        Self::write_text_to_path(&path, &self.generate_file_content()).map(|()| path)
    }

    /// Saves the crash report to a file and
    /// prints about whether and where it saved.
    ///
    /// Returns `true` if the file successfully saved.
    pub fn save_and_log(&self) -> bool {
        match self.save() {
            Ok(path) => {
                tracing::info!(
                    "{} {}",
                    Color::Named(NamedColor::Green)
                        .console_color("Successfully saved the crash report to file:"),
                    path.display()
                );
                true
            }
            Err(error) => {
                tracing::error!(
                    "{} {}",
                    Color::Named(NamedColor::Red).console_color("Could not save the crash report:"),
                    error
                );
                false
            }
        }
    }

    fn write_text_to_path(path: &Path, text: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let mut file = File::create_new(path)?;
        <File as std::io::Write>::write_all(&mut file, text.as_bytes())?;

        Ok(())
    }

    /// Convenient method to return a reference to the report's full backtrace.
    const fn full_backtrace(&self) -> &Backtrace {
        match &self.full_backtrace {
            FullBacktrace::Captured => &self.captured_backtrace,
            FullBacktrace::ForceCaptured(backtrace) => backtrace,
        }
    }
}
