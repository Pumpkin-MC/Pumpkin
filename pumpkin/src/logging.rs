use flate2::write::GzEncoder;
use log::{LevelFilter, Log};
use rustyline_async::Readline;
use simplelog::{CombinedLogger, Config, SharedLogger, WriteLogger};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

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
        std::fs::create_dir_all("logs")?;

        // If latest.log exists, we will gzip it
        if Path::new(&format!("logs/{filename}")).exists() {
            let new_filename = Self::new_filename(false);
            let mut file = File::open(format!("logs/{filename}"))?;
            let mut encoder = GzEncoder::new(
                BufWriter::new(File::create(&new_filename)?),
                flate2::Compression::best(),
            );
            println!("logs/{filename}");
            std::io::copy(&mut file, &mut encoder)?;
            encoder.finish()?;
        }

        Ok(Box::new(Self {
            log_level,
            data: std::sync::Mutex::new(GzipRollingLoggerData {
                current_day_of_month: now.day(),
                last_rotate_time: now,
                latest_filename: filename.clone(),
                latest_logger: *WriteLogger::new(
                    log_level,
                    config.clone(),
                    File::create(format!("logs/{filename}")).unwrap(),
                ),
            }),
            config,
        }))
    }

    pub fn new_filename(yesterday: bool) -> String {
        let mut now = time::OffsetDateTime::now_utc()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC));
        if yesterday {
            now -= time::Duration::days(1)
        }
        let base_filename = format!("{}-{:02}-{:02}", now.year(), now.month() as u8, now.day());

        let mut id = 1;
        loop {
            let filename = format!("logs/{base_filename}-{id}.log.gz");
            if !Path::new(&filename).exists() {
                return filename;
            }
            id += 1;
        }
    }

    fn rotate_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = time::OffsetDateTime::now_utc();
        let mut data = self.data.lock().unwrap();

        let new_filename = Self::new_filename(true);
        let mut file = File::open(format!("logs/{}", data.latest_filename))?;
        let mut encoder = GzEncoder::new(
            BufWriter::new(File::create(format!("logs/{new_filename}"))?),
            flate2::Compression::best(),
        );
        std::io::copy(&mut file, &mut encoder)?;
        encoder.finish()?;

        data.current_day_of_month = now.day();
        data.last_rotate_time = now;
        data.latest_logger = *WriteLogger::new(
            self.log_level,
            self.config.clone(),
            File::create(format!("logs/{}", data.latest_filename)).unwrap(),
        );
        Ok(())
    }
}

impl Log for GzipRollingLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let now = time::OffsetDateTime::now_utc();

        if let Ok(data) = self.data.lock() {
            data.latest_logger.log(record);
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

    pub(crate) fn take_readline(&self) -> Option<Readline> {
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
        if let Ok(mut lock) = self.readline.lock() {
            if let Some(rl) = lock.as_mut() {
                let _ = rl.flush();
            }
        }
    }

    fn flush(&self) {
        self.internal.flush();
        if let Ok(mut lock) = self.readline.lock() {
            if let Some(rl) = lock.as_mut() {
                let _ = rl.flush();
            }
        }
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.internal.enabled(metadata)
    }
}
