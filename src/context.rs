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

    pub fn scroll_down(&mut self) -> anyhow::Result<bool> {
        let old_offset = self.offset;
        self.offset += 1;
        if self.buffer.len() - self.offset < self.height - 1 {
            let line = self.buffer.append_line()?;
            if line.is_none() {
                // TODO: Display an END marker
                self.offset -= 1;
            }
        }
        Ok(old_offset != self.offset)
    }

    pub fn sroll_up(&mut self) -> anyhow::Result<bool> {
        let old_offset = self.offset;
        self.offset = self.offset.saturating_sub(1);
        Ok(old_offset != self.offset)
    }
}
