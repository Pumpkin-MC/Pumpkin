use flate2::write::GzEncoder;
use log::{LevelFilter, Log, Record};
use rustyline_async::Readline;
use simplelog::{CombinedLogger, Config, SharedLogger, WriteLogger};
use std::fmt::format;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use time::{Duration, OffsetDateTime, UtcOffset};

const LOG_DIR: &str = "logs";
const MAX_ATTEMPTS: u32 = 100;

/// A wrapper for our logger to hold the terminal input while no input is expected in order to
/// properly flush logs to the output while they happen instead of batched
pub struct ReadlineLogWrapper {
    internal: Box<CombinedLogger>,
    readline: std::sync::Mutex<Option<Readline>>,
}

struct GzipRollingLoggerData {
    pub current_day_of_month: u8,
    pub last_rotate_time: time::OffsetDateTime,
    pub latest_logger: WriteLogger<File>,
    latest_filename: String,
}

pub struct GzipRollingLogger {
    log_level: LevelFilter,
    data: std::sync::Mutex<GzipRollingLoggerData>,
    config: Config,
}

impl GzipRollingLogger {
    pub fn new(
        log_level: LevelFilter,
        config: Config,
        filename: String,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        let now = time::OffsetDateTime::now_utc();
        std::fs::create_dir_all(LOG_DIR)?;

        let latest_path = PathBuf::from(LOG_DIR).join(&filename);

        // If latest.log exists, we will gzip it
        if latest_path.exists() {
            eprintln!(
                "Found existing log file at '{}', gzipping it now...",
                latest_path.display()
            );

            let new_gz_path = Self::new_filename(true)?;

            let mut file = File::open(&latest_path)?;

            let mut encoder = GzEncoder::new(
                BufWriter::new(File::create(&new_gz_path)?),
                flate2::Compression::best(),
            );

            io::copy(&mut file, &mut encoder)?;
            encoder.finish()?;

            std::fs::remove_file(&latest_path)?;
        }

        let new_logger = WriteLogger::new(log_level, config.clone(), File::create(&latest_path)?);

        Ok(Box::new(Self {
            log_level,
            data: std::sync::Mutex::new(GzipRollingLoggerData {
                current_day_of_month: now.day(),
                last_rotate_time: now,
                latest_filename: filename,
                latest_logger: *new_logger,
            }),
            config,
        }))
    }

    pub fn new_filename(yesterday: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
        let mut now = OffsetDateTime::now_utc().to_offset(local_offset);

        if yesterday {
            now -= Duration::days(1);
        }

        let date_format = format!("{}-{:02}-{:02}", now.year(), now.month() as u8, now.day());

        let log_path = PathBuf::from(LOG_DIR);

        for id in 1..=MAX_ATTEMPTS {
            let filename = log_path.join(format!("{}-{}.log.gz", date_format, id));

            if !filename.exists() {
                return Ok(filename);
            }
        }

        Err(format!(
            "Failed to find a unique log filename for date {} after {} attempts.",
            date_format, MAX_ATTEMPTS
        )
        .into())
    }

    fn rotate_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = time::OffsetDateTime::now_utc();
        let mut data = self.data.lock().unwrap();

        let new_gz_path = Self::new_filename(true)?;
        let latest_path = PathBuf::from(LOG_DIR).join(&data.latest_filename);
        let mut file = File::open(&latest_path)?;
        let mut encoder = GzEncoder::new(
            BufWriter::new(File::create(&new_gz_path)?),
            flate2::Compression::best(),
        );
        io::copy(&mut file, &mut encoder)?;
        encoder.finish()?;

        data.current_day_of_month = now.day();
        data.last_rotate_time = now;
        data.latest_logger = *WriteLogger::new(
            self.log_level,
            self.config.clone(),
            File::create(&latest_path)?,
        );
        Ok(())
    }
}

fn remove_ansi_color_code(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut it = s.chars();

    while let Some(c) = it.next() {
        if c == '\x1b' {
            for c_seq in it.by_ref() {
                if c_seq.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

impl Log for GzipRollingLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let now = time::OffsetDateTime::now_utc();

        if let Ok(data) = self.data.lock() {
            let original_string = format(*record.args());
            let string = remove_ansi_color_code(&original_string);
            data.latest_logger.log(
                &Record::builder()
                    .args(format_args!("{string}"))
                    .metadata(record.metadata().clone())
                    .module_path(record.module_path())
                    .file(record.file())
                    .line(record.line())
                    .build(),
            );
            if data.current_day_of_month != now.day() {
                drop(data);
                if let Err(e) = self.rotate_log() {
                    eprintln!("Failed to rotate log: {e}");
                }
            }
        }
    }

    fn flush(&self) {
        if let Ok(data) = self.data.lock() {
            data.latest_logger.flush();
        }
    }
}

impl SharedLogger for GzipRollingLogger {
    fn level(&self) -> LevelFilter {
        self.log_level
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}

impl ReadlineLogWrapper {
    pub fn new(
        log: Box<dyn SharedLogger + 'static>,
        file_logger: Option<Box<dyn SharedLogger + 'static>>,
        rl: Option<Readline>,
    ) -> Self {
        let loggers: Vec<Option<Box<dyn SharedLogger + 'static>>> = vec![Some(log), file_logger];
        Self {
            internal: CombinedLogger::new(loggers.into_iter().flatten().collect()),
            readline: std::sync::Mutex::new(rl),
        }
    }

    pub fn take_readline(&self) -> Option<Readline> {
        if let Ok(mut result) = self.readline.lock() {
            result.take()
        } else {
            None
        }
    }

    pub(crate) fn return_readline(&self, rl: Readline) {
        if let Ok(mut result) = self.readline.lock() {
            println!("Returned rl");
            let _ = result.insert(rl);
        }
    }
}

// Writing to `stdout` is expensive anyway, so I don't think having a `Mutex` here is a big deal.
impl Log for ReadlineLogWrapper {
    fn log(&self, record: &log::Record) {
        self.internal.log(record);
        if let Ok(mut lock) = self.readline.lock()
            && let Some(rl) = lock.as_mut()
        {
            let _ = rl.flush();
        }
    }

    fn flush(&self) {
        self.internal.flush();
        if let Ok(mut lock) = self.readline.lock()
            && let Some(rl) = lock.as_mut()
        {
            let _ = rl.flush();
        }
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.internal.enabled(metadata)
    }
}
