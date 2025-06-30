use flate2::write::GzEncoder;
use log::{LevelFilter, Log};
use rustyline_async::Readline;
use simplelog::{CombinedLogger, Config, SharedLogger, WriteLogger};
use std::fs::File;
use std::io::{BufReader, BufWriter};
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
    pub gz_logger: WriteLogger<GzEncoder<BufWriter<File>>>,
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
    ) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        let now = time::OffsetDateTime::now_utc();

        Ok(Box::new(Self {
            log_level,
            data: std::sync::Mutex::new(GzipRollingLoggerData {
                current_day_of_month: now.day(),
                last_rotate_time: now,
                gz_logger: *WriteLogger::new(
                    log_level,
                    config.clone(),
                    GzEncoder::new(
                        BufWriter::new(File::create(Self::new_filename()).unwrap()),
                        flate2::Compression::default(),
                    ),
                ),
            }),
            config,
        }))
    }

    pub fn new_filename() -> String {
        let now = time::OffsetDateTime::now_utc();
        let base_filename = format!("{}-{:02}-{:02}", now.year(), now.month() as u8, now.day());

        // 查找唯一的文件名
        let mut id = 1;
        loop {
            let filename = format!("{}-{}.log.gz", base_filename, id);
            if !Path::new(&filename).exists() {
                return filename;
            }
            id += 1;
        }
    }

    fn rotate_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = time::OffsetDateTime::now_utc();
        let mut data = self.data.lock().unwrap();

        data.current_day_of_month = now.day();
        data.last_rotate_time = now;
        data.gz_logger = *WriteLogger::new(
            self.log_level,
            self.config.clone(),
            GzEncoder::new(
                BufWriter::new(File::create(Self::new_filename()).unwrap()),
                flate2::Compression::default(),
            ),
        );
        Ok(())
    }

    fn format_log_record(&self, record: &log::Record) -> String {
        let now = time::OffsetDateTime::now_utc();
        format!(
            "[{} {} {}:{}] {}\n",
            now.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "UNKNOWN_TIME".to_string()),
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
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

        {
            let data = self.data.lock().unwrap();
            if data.current_day_of_month != now.day() {
                drop(data);
                if let Err(e) = self.rotate_log() {
                    eprintln!("Failed to rotate log: {}", e);
                    return;
                }
            }
        }

        if let Ok(mut data) = self.data.lock() {
            data.gz_logger.log(record)
        }
    }

    fn flush(&self) {
        if let Ok(data) = self.data.lock() {
            data.gz_logger.flush();
        }
    }
}

impl ReadlineLogWrapper {
    fn new(
        log: Box<dyn SharedLogger + 'static>,
        file_logger: Option<Box<dyn SharedLogger + 'static>>,
        gzip_logger: Option<Box<dyn SharedLogger + 'static>>,
        rl: Option<Readline>,
    ) -> Self {
        let loggers: Vec<Option<Box<dyn SharedLogger + 'static>>> =
            vec![Some(log), file_logger, gzip_logger];
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

    fn return_readline(&self, rl: Readline) {
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
