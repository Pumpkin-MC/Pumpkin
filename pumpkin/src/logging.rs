use log4rs::append::Append;
use log4rs::config::Deserializers;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::{Color, Encode, EncoderConfig, Style};
use rustyline_async::{Readline, SharedWriter};
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::sync::{Arc, Mutex};

pub static LOGGER_TIME_FORMAT: &[time::format_description::BorrowedFormatItem] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub struct RustylineConsoleAppender {
    encoder: Box<dyn Encode>,
    pub readline: Arc<Mutex<Option<Readline>>>,
    pub writer: Arc<Mutex<PumpkinLogWriter>>,
}

pub enum PumpkinLogWriter {
    Tty(SharedWriter),
    Raw(std::io::Stdout),
}

impl Write for PumpkinLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            PumpkinLogWriter::Tty(tty) => std::io::Write::write(tty, buf),
            PumpkinLogWriter::Raw(raw) => raw.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            PumpkinLogWriter::Tty(tty) => std::io::Write::flush(tty),
            PumpkinLogWriter::Raw(raw) => raw.flush(),
        }
    }
}

fn color_byte(c: Color) -> u8 {
    match c {
        Color::Black => b'0',
        Color::Red => b'1',
        Color::Green => b'2',
        Color::Yellow => b'3',
        Color::Blue => b'4',
        Color::Magenta => b'5',
        Color::Cyan => b'6',
        Color::White => b'7',
    }
}

impl log4rs::encode::Write for PumpkinLogWriter {
    fn set_style(&mut self, style: &Style) -> std::io::Result<()> {
        match self {
            PumpkinLogWriter::Tty(tty) => {
                let mut buf = [0; 12];
                buf[0] = b'\x1b';
                buf[1] = b'[';
                buf[2] = b'0';
                let mut idx = 3;

                if let Some(text) = style.text {
                    buf[idx] = b';';
                    buf[idx + 1] = b'3';
                    buf[idx + 2] = color_byte(text);
                    idx += 3;
                }

                if let Some(background) = style.background {
                    buf[idx] = b';';
                    buf[idx + 1] = b'4';
                    buf[idx + 2] = color_byte(background);
                    idx += 3;
                }

                if let Some(intense) = style.intense {
                    buf[idx] = b';';
                    if intense {
                        buf[idx + 1] = b'1';
                        idx += 2;
                    } else {
                        buf[idx + 1] = b'2';
                        buf[idx + 2] = b'2';
                        idx += 3;
                    }
                }
                buf[idx] = b'm';
                Write::write_all(tty, &buf[..=idx])
            }
            PumpkinLogWriter::Raw(_) => Ok(()),
        }
    }
}

impl RustylineConsoleAppender {
    pub fn new(
        encoder: Box<dyn Encode>,
        readline: Arc<Mutex<Option<Readline>>>,
        writer: Arc<Mutex<PumpkinLogWriter>>,
    ) -> Self {
        Self {
            encoder,
            readline,
            writer,
        }
    }
}

impl Debug for RustylineConsoleAppender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustylineConsoleAppender").finish()
    }
}

impl Append for RustylineConsoleAppender {
    fn append(&self, record: &log::Record) -> anyhow::Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock writer mutex"))?;
        self.encoder.encode(&mut *writer, record)?;
        drop(writer);
        if let Some(readline) = self
            .readline
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock readline mutex"))?
            .as_mut()
        {
            readline.flush()?;
        }
        Ok(())
    }

    fn flush(&self) {}
}

#[derive(Debug, serde::Deserialize)]
struct RustylineConsoleAppenderConfig {
    encoder: Option<EncoderConfig>,
}

struct RustylineConsoleAppenderDeserializer {
    readline: Arc<Mutex<Option<Readline>>>,
    writer: Arc<Mutex<PumpkinLogWriter>>,
}

impl log4rs::config::Deserialize for RustylineConsoleAppenderDeserializer {
    type Trait = dyn Append;
    type Config = RustylineConsoleAppenderConfig;

    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let encoder = if let Some(encoder_config) = config.encoder {
            deserializers.deserialize(&encoder_config.kind, encoder_config.config)?
        } else {
            Box::<PatternEncoder>::default() as Box<dyn Encode>
        };

        Ok(Box::new(RustylineConsoleAppender::new(
            encoder,
            self.readline.clone(),
            self.writer.clone(),
        )) as Box<dyn Append>)
    }
}

pub fn register_rustyline_console_appender(
    deserializers: &mut Deserializers,
    readline: Arc<Mutex<Option<Readline>>>,
    writer: Arc<Mutex<PumpkinLogWriter>>,
) {
    deserializers.insert(
        "rustyline_console",
        RustylineConsoleAppenderDeserializer { readline, writer },
    );
}
