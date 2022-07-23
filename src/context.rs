use std::io::{Read, Write};

use log::debug;

use crate::buffer::{self, Buffer};

pub struct Context<R> {
    buffer: Buffer<R>,
    width: usize,
    height: usize,
    offset: usize,
}

impl<R> Context<R> {
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.buffer
            .lines(self.width)
            .skip(self.offset)
            .take(self.height - 1)
    }

    pub fn write_screen<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        debug!("Refreshing screen");
        write!(writer, "{}", termion::cursor::Goto(1, 1))?;
        for line in self.lines() {
            write!(writer, "{}{}\n\r", termion::clear::CurrentLine, line)?;
        }
        write!(writer, ":")?;
        writer.flush()?;
        Ok(())
    }
}

impl<R> Context<R>
where
    R: Read,
{
    pub fn new(width: u16, height: u16, reader: R) -> Self {
        debug!("Width: {}, Height: {}", width, height);
        Self {
            buffer: Buffer::new(reader),
            width: width as usize,
            height: height as usize,
            offset: 0,
        }
    }

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

    pub fn scroll_down(&mut self, lines: usize) -> anyhow::Result<bool> {
        let old_offset = self.offset;
        self.offset += lines;
        while self.buffer.len() - self.offset < self.height - 1 {
            let line = self.buffer.append_line()?;
            if line.is_none() {
                // TODO: Display an END marker
                self.offset = self.buffer.len() - self.height + 1;
                break;
            }
        }
        Ok(old_offset != self.offset)
    }

    pub fn scroll_up(&mut self, lines: usize) -> anyhow::Result<bool> {
        let old_offset = self.offset;
        self.offset = self.offset.saturating_sub(lines);
        Ok(old_offset != self.offset)
    }

    pub fn scroll_down_line(&mut self) -> anyhow::Result<bool> {
        self.scroll_down(1)
    }

    pub fn scroll_up_line(&mut self) -> anyhow::Result<bool> {
        self.scroll_up(1)
    }

    pub fn scroll_down_screen(&mut self) -> anyhow::Result<bool> {
        self.scroll_down(self.height - 1)
    }

    pub fn scroll_up_screen(&mut self) -> anyhow::Result<bool> {
        self.scroll_up(self.height - 1)
    }
}
