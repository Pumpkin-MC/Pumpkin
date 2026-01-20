#![allow(clippy::print_stderr)]
#![allow(clippy::print_stdout)]

use flate2::write::GzEncoder;
use log::{LevelFilter, Log, Record};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};
use simplelog::{CombinedLogger, Config, SharedLogger, WriteLogger};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::format;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use time::{Duration, OffsetDateTime, UtcOffset};

use crate::command::CommandSender;
use crate::command::tree::NodeType;
use crate::server::Server;

const LOG_DIR: &str = "logs";
const MAX_ATTEMPTS: u32 = 100;

/// A wrapper for our logger to hold the terminal input while no input is expected in order to
/// properly flush logs to the output while they happen instead of batched
pub struct ReadlineLogWrapper {
    internal: Box<CombinedLogger>,
    readline: std::sync::Mutex<Option<Editor<PumpkinCommandCompleter, FileHistory>>>,
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
            let filename = log_path.join(format!("{date_format}-{id}.log.gz"));

            if !filename.exists() {
                return Ok(filename);
            }
        }

        Err(format!(
            "Failed to find a unique log filename for date {date_format} after {MAX_ATTEMPTS} attempts.",
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

impl ReadlineLogWrapper {
    #[must_use]
    pub fn new(
        log: Box<dyn SharedLogger + 'static>,
        file_logger: Option<Box<dyn SharedLogger + 'static>>,
        rl: Option<Editor<PumpkinCommandCompleter, FileHistory>>,
    ) -> Self {
        let loggers: Vec<Option<Box<dyn SharedLogger + 'static>>> = vec![Some(log), file_logger];
        Self {
            internal: CombinedLogger::new(loggers.into_iter().flatten().collect()),
            readline: std::sync::Mutex::new(rl),
        }
    }

    pub fn take_readline(&self) -> Option<Editor<PumpkinCommandCompleter, FileHistory>> {
        self.readline
            .lock()
            .map_or_else(|_| None, |mut result| result.take())
    }

    #[expect(dead_code)]
    pub(crate) fn return_readline(&self, rl: Editor<PumpkinCommandCompleter, FileHistory>) {
        if let Ok(mut result) = self.readline.lock() {
            let _ = result.insert(rl);
        }
    }
}

// Writing to `stdout` is expensive anyway, so I don't think having a `Mutex` here is a big deal.
impl Log for ReadlineLogWrapper {
    fn log(&self, record: &log::Record) {
        self.internal.log(record);
    }

    fn flush(&self) {
        self.internal.flush();
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.internal.enabled(metadata)
    }
}

#[derive(Clone, Default)]
pub struct PumpkinCommandCompleter {
    pub server: Arc<std::sync::RwLock<Option<Arc<Server>>>>,
    pub rt: Arc<std::sync::OnceLock<tokio::runtime::Handle>>,
}

impl PumpkinCommandCompleter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            server: Arc::new(std::sync::RwLock::new(None)),
            rt: Arc::new(std::sync::OnceLock::new()),
        }
    }
}

impl Helper for PumpkinCommandCompleter {}
impl Highlighter for PumpkinCommandCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        line.find(' ').map_or_else(
            || Cow::Owned(format!("\x1b[1;36m{line}\x1b[0m")),
            |first_space| {
                let (cmd, args) = line.split_at(first_space);
                Cow::Owned(format!("\x1b[1;36m{cmd}\x1b[0m{args}"))
            },
        )
    }
}
impl Hinter for PumpkinCommandCompleter {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        if let Ok((_, candidates)) = self.complete(line, pos, ctx)
            && let Some(first) = candidates.first()
        {
            let last_word = line.split_whitespace().last().unwrap_or("");
            if first.starts_with('<') {
                return line.ends_with(' ').then(|| first.clone());
            }

            if let Some(stripped) = first.strip_prefix(last_word) {
                return Some(stripped.to_owned());
            }
        }
        None
    }
}

impl Validator for PumpkinCommandCompleter {}

impl Completer for PumpkinCommandCompleter {
    type Candidate = String;

    #[expect(clippy::too_many_lines)]
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let cmd_to_cursor = &line[..pos];
        let has_slash = cmd_to_cursor.starts_with('/');
        let cmd = if has_slash {
            &cmd_to_cursor[1..]
        } else {
            cmd_to_cursor
        };

        let Some(handle) = self.rt.get() else {
            return Ok((0, Vec::new()));
        };
        let Ok(server_guard) = self.server.try_read() else {
            return Ok((0, Vec::new()));
        };
        let Some(server) = server_guard.as_ref() else {
            return Ok((0, Vec::new()));
        };

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let ends_with_space = cmd.ends_with(' ');

        handle.block_on(async {
            let dispatcher = server.command_dispatcher.read().await;
            let src = CommandSender::Console;

            if parts.is_empty() || (parts.len() == 1 && !ends_with_space) {
                let typing = parts.first().unwrap_or(&"");
                let candidates = dispatcher
                    .commands
                    .keys()
                    .filter(|k| k.starts_with(typing))
                    .cloned()
                    .collect();
                return Ok((usize::from(has_slash), candidates));
            }

            let Some(tree) = dispatcher.get_tree(parts[0]).ok() else {
                return Ok((0, Vec::new()));
            };

            let mut current_indices = tree.children.clone();
            let mut word_index = 1;
            let walk_limit = if ends_with_space {
                parts.len()
            } else {
                parts.len() - 1
            };

            while word_index < walk_limit {
                let token = parts[word_index];
                let mut next_indices = Vec::new();

                let mut worklist: VecDeque<usize> = current_indices.iter().copied().collect();

                while let Some(idx) = worklist.pop_front() {
                    let node = &tree.nodes[idx];

                    match &node.node_type {
                        NodeType::Require { predicate } => {
                            if predicate(&src) {
                                worklist.extend(node.children.iter().copied());
                            }
                        }
                        NodeType::Literal { string } => {
                            if string.eq_ignore_ascii_case(token) {
                                next_indices.extend(node.children.iter().copied());
                            }
                        }
                        NodeType::Argument { .. } => {
                            next_indices.extend(node.children.iter().copied());
                        }
                        NodeType::ExecuteLeaf { .. } => {}
                    }
                }

                if next_indices.is_empty() {
                    return Ok((0, Vec::new()));
                }

                current_indices = next_indices;
                word_index += 1;
            }

            let typing = if ends_with_space {
                ""
            } else {
                parts.last().unwrap_or(&"")
            };
            let mut candidates = Vec::new();

            let mut suggestion_worklist: VecDeque<usize> = current_indices.into_iter().collect();

            while let Some(idx) = suggestion_worklist.pop_front() {
                let node = &tree.nodes[idx];
                match &node.node_type {
                    NodeType::Require { predicate } => {
                        if predicate(&src) {
                            suggestion_worklist.extend(node.children.iter().copied());
                        }
                    }
                    NodeType::Literal { string } => {
                        if string.starts_with(typing) {
                            candidates.push(string.clone());
                        }
                    }
                    NodeType::Argument { name, consumer } => {
                        let suggest_future = consumer.suggest(&src, server, typing);

                        if let Ok(Some(suggestions)) = suggest_future.await {
                            for s in suggestions {
                                let s = s.suggestion;
                                if s.starts_with(typing) {
                                    candidates.push(s);
                                }
                            }
                        } else {
                            let placeholder = format!("<{name}>");
                            if placeholder.starts_with(typing) || typing.is_empty() {
                                candidates.push(placeholder);
                            }
                        }
                    }
                    NodeType::ExecuteLeaf { executor: _ } => {}
                }
            }

            let last_space = cmd.rfind(' ').map_or(0, |i| i + 1);
            Ok((last_space + usize::from(has_slash), candidates))
        })
    }
}
