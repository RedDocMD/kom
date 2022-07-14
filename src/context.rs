use std::io::{Read, Write};

use crate::buffer::{self, Buffer};

pub struct Context<R> {
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

    pub fn write_screen<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        for line in self.lines() {
            write!(writer, "{}\n\r", line)?;
        }
        Ok(())
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
