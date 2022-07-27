use std::{
    env,
    fs::File,
    io::{self, stdin, stdout, Read},
    path::PathBuf,
    process,
};

use clap::Parser;
use log::LevelFilter;
use rand::Rng;
use simplelog::WriteLogger;
use termion::{get_tty, raw::IntoRawMode, screen::AlternateScreen};

use crate::command::CommandDispatcher;

use self::context::Context;

mod buffer;
mod command;
mod context;

#[macro_use]
extern crate log;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser)]
    filename: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let log_file = LogFile::new()?;
    let level = parse_level(&env::var("KOM_LOG_LEVEL").unwrap_or_else(|_| "info".into()));
    WriteLogger::init(level, simplelog::Config::default(), log_file)?;

    debug!("Initialized logger");

    if !termion::is_tty(&stdout()) {
        eprintln!("Expected stdout to be a TTY!");
        process::exit(1);
    }

    let cli = Cli::parse();

    let (width, height) = termion::terminal_size()?;

    let screen = AlternateScreen::from(stdout());
    let mut raw_screen = screen.into_raw_mode()?;

    let input = if let Some(filename) = &cli.filename {
        let file = File::open(filename)?;
        Box::new(file) as Box<dyn Read>
    } else {
        Box::new(stdin()) as Box<dyn Read>
    };

    let filename = cli.filename.map(|name| name.display().to_string());

    let mut ctx = Context::new(width, height, input, filename);
    ctx.fill_buffer()?;

    ctx.write_screen(&mut raw_screen)?;

    let tty = get_tty()?;
    let mut cmd_dispatcher = CommandDispatcher::new(&mut ctx, raw_screen);
    cmd_dispatcher.handle_events(tty)?;

    Ok(())
}

fn log_file() -> PathBuf {
    let mut path = PathBuf::from("/tmp");
    let mut rng = rand::thread_rng();
    let tail: String = (0..5).map(|_| rng.gen_range('a'..'z')).collect();
    path.push(format!("kom.log.{}", tail));
    path
}

struct LogFile {
    file: File,
}

impl LogFile {
    pub fn new() -> anyhow::Result<Self> {
        let file = File::create(log_file())?;
        Ok(Self { file })
    }
}

impl io::Write for LogFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let cnt = self.file.write(buf)?;
        self.flush()?;
        Ok(cnt)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn parse_level(level: &str) -> LevelFilter {
    match level {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}
