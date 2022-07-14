use std::{
    io::{stdin, stdout},
    process,
};

use termion::{raw::IntoRawMode, screen::AlternateScreen};

use self::context::Context;

mod buffer;
mod context;

fn main() -> anyhow::Result<()> {
    if !termion::is_tty(&stdout()) {
        println!("Expected stdout to be a TTY!");
        process::exit(1);
    }

    let (width, height) = termion::terminal_size()?;

    let screen = AlternateScreen::from(stdout());
    let mut raw_screen = screen.into_raw_mode()?;

    let mut context = Context::new(width, height, stdin());
    context.fill_buffer()?;
    context.write_screen(&mut raw_screen)?;

    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}
