use std::{
    io::{stdin, stdout, Read, Write},
    process,
};

use termion::raw::IntoRawMode;

use self::buffer::Buffer;

mod buffer;

struct Context<R> {
    buffer: Buffer<R>,
    width: usize,
    height: usize,
    offset: usize,
}

impl<R> Context<R> {
    pub fn new(width: u16, height: u16, reader: R) -> Self {
        Self {
            buffer: Buffer::new(reader),
            width: width as usize,
            height: height as usize,
            offset: 0,
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.buffer.lines(self.width).take(self.height - 1)
    }
}

impl<R> Context<R>
where
    R: Read,
{
    pub fn fill_buffer(&mut self) -> anyhow::Result<()> {
        let mut cnt = 0;
        while cnt < self.height - 1 {
            if let Some(line) = self.buffer.append_line()? {
                cnt += buffer::line_width_divisions(line, self.width);
            } else {
                break;
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    if !termion::is_tty(&stdout()) {
        println!("Expected stdout to be a TTY!");
        process::exit(1);
    }

    let (width, height) = termion::terminal_size()?;
    let mut context = Context::new(width, height, stdin());

    let mut raw_stdout = stdout().into_raw_mode()?;
    context.fill_buffer()?;
    for line in context.lines() {
        write!(raw_stdout, "{}\n\r", line)?;
    }

    Ok(())
}
