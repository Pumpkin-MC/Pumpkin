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

use crate::{localized_log, localized_log_format};

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
            TextComponent::text(localized_log("crash.encountered_panic"))
                .color(RED)
                .bold()
                .to_pretty_console()
        );

        error!("");

        // Printing panic info.
        let thread_name = self
            .thread
            .name()
            .map_or_else(|| localized_log("crash.unnamed_thread"), ToOwned::to_owned);
        let thread_id = self.thread.id();

        let message = self.panic_location.as_ref().map_or_else(
            || {
                localized_log_format(
                    "crash.thread_panicked",
                    &[thread_name.clone(), format!("{thread_id:?}")],
                )
            },
            |location| {
                localized_log_format(
                    "crash.thread_panicked_at",
                    &[
                        thread_name.clone(),
                        format!("{thread_id:?}"),
                        location.to_string(),
                    ],
                )
            },
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
                    RED.console_color(&localized_log("crash.backtracing_unsupported"))
                );
            }
            // It cannot possibly be BacktraceStatus::Disabled
            // as it is a forced backtrace.
            BacktraceStatus::Captured => {
                error!(
                    "{}",
                    RED.console_color(&localized_log("crash.full_backtrace_will_print"))
                );

                if self.captured_backtrace.status() == BacktraceStatus::Captured {
                    eprintln!(
                        "{}\n{}",
                        RED.console_color(&localized_log("crash.backtrace_label")),
                        self.captured_backtrace
                    );
                }
            }
            _ => {
                error!(
                    "{}",
                    RED.console_color(&localized_log("server.log.backtrace_unknown"))
                );
            }
        }
    }

    /// Generates the file content of the crash report file
    /// that would be generated from this report.
    pub fn generate_file_content(&self) -> String {
        let mut output = String::new();

        writeln_output!(&mut output, "{}", localized_log("crash.report.title"));
        writeln_output!(&mut output);
        writeln_output!(
            &mut output,
            "{}",
            localized_log_format("crash.report.time", &[self.utc_time.to_string()])
        );
        writeln_output!(
            &mut output,
            "{}",
            localized_log_format(
                "crash.report.message",
                &[self
                    .payload
                    .clone()
                    .unwrap_or_else(|| localized_log("crash.unknown_payload"))],
            )
        );

        if let Some(panic_location) = &self.panic_location {
            writeln_output!(
                &mut output,
                "{}",
                localized_log_format("crash.report.panic_location", &[panic_location.to_string()],)
            );
        }

        writeln_output!(&mut output);
        writeln_output!(
            &mut output,
            "{}",
            localized_log("crash.report.section_panicking_thread")
        );
        writeln_output!(
            &mut output,
            "{}",
            localized_log_format(
                "crash.report.thread_id",
                &[format!("{:?}", self.thread.id())]
            )
        );
        if let Some(thread_name) = self.thread.name() {
            writeln_output!(
                &mut output,
                "{}",
                localized_log_format("crash.report.thread_name", &[thread_name.to_owned()])
            );
        }
        writeln_output!(&mut output, "{}", localized_log("crash.backtrace_label"));
        writeln_output!(&mut output, "{}", self.full_backtrace());

        writeln_output!(
            &mut output,
            "{}",
            localized_log("crash.report.section_server_details")
        );

        writeln_output!(
            &mut output,
            "{}",
            localized_log_format(
                "crash.report.pumpkin_version",
                &[Self::get_pumpkin_version()]
            )
        );
        writeln_output!(
            &mut output,
            "{}",
            localized_log_format(
                "crash.report.minecraft_version",
                &[CURRENT_MC_VERSION.to_string()],
            )
        );
        writeln_output!(
            &mut output,
            "{}",
            localized_log_format(
                "crash.report.rust_version",
                &[rustc_version_runtime::version().to_string()],
            )
        );

        if sysinfo::IS_SUPPORTED_SYSTEM {
            writeln_output!(&mut output);
            writeln_output!(
                &mut output,
                "{}",
                localized_log("crash.report.section_system_details")
            );

            let mut sys = System::new_all();
            sys.refresh_all();

            writeln_output!(
                &mut output,
                "{}",
                localized_log_format(
                    "crash.report.operating_system",
                    &[System::long_os_version()
                        .unwrap_or_else(|| localized_log("crash.report.unknown"))],
                )
            );
            writeln_output!(
                &mut output,
                "{}",
                localized_log_format(
                    "crash.report.kernel",
                    &[System::kernel_version()
                        .unwrap_or_else(|| localized_log("crash.report.unknown"))],
                )
            );
            writeln_output!(
                &mut output,
                "{}",
                localized_log_format(
                    "crash.report.physical_memory",
                    &[
                        (sys.used_memory() / BYTES_PER_MEBIBYTE).to_string(),
                        (sys.total_memory() / BYTES_PER_MEBIBYTE).to_string(),
                        (sys.free_memory() / BYTES_PER_MEBIBYTE).to_string(),
                    ],
                )
            );
            writeln_output!(
                &mut output,
                "{}",
                localized_log_format(
                    "crash.report.swap_memory",
                    &[
                        (sys.used_swap() / BYTES_PER_MEBIBYTE).to_string(),
                        (sys.total_swap() / BYTES_PER_MEBIBYTE).to_string(),
                        (sys.free_swap() / BYTES_PER_MEBIBYTE).to_string(),
                    ],
                )
            );

            Self::write_cpus(&mut output, &sys);
        }

        output
    }

    fn write_cpus(output: &mut String, sys: &System) {
        writeln_output!(output);
        let cpus = sys.cpus();

        writeln_output!(
            output,
            "{}",
            localized_log_format("crash.report.total_cores", &[cpus.len().to_string()])
        );
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

            writeln_output!(
                output,
                "{}",
                localized_log_format(
                    "crash.report.cpu_cores",
                    &[prefix.clone(), cpus.len().to_string()],
                )
            );
            writeln_output!(
                output,
                "{}",
                localized_log_format("crash.report.cpu_names", &[padded.clone(), names])
            );
            writeln_output!(
                output,
                "{}",
                localized_log_format(
                    "crash.report.cpu_brand",
                    &[padded.clone(), brand.to_string()]
                )
            );
            writeln_output!(
                output,
                "{}",
                localized_log_format(
                    "crash.report.cpu_average_frequency",
                    &[padded.clone(), avg_freq.to_string()],
                )
            );
            writeln_output!(
                output,
                "{}",
                localized_log_format(
                    "crash.report.cpu_vendor_id",
                    &[padded, vendor_id.to_string()]
                )
            );
            writeln_output!(output);
        }
    }

    #[must_use]
    pub fn get_pumpkin_version() -> String {
        let profile = if cfg!(debug_assertions) {
            localized_log("server.log.profile_debug")
        } else {
            localized_log("server.log.profile_release")
        };
        localized_log_format(
            "crash.report.pumpkin_version_value",
            &[
                env!("CARGO_PKG_VERSION").to_owned(),
                env!("GIT_HASH").to_owned(),
                profile,
            ],
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
                        .console_color(&localized_log("crash.saved_crash_report")),
                    path.display()
                );
                true
            }
            Err(error) => {
                tracing::error!(
                    "{} {}",
                    Color::Named(NamedColor::Red)
                        .console_color(&localized_log("crash.could_not_save_crash_report")),
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
