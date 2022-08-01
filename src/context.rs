use std::io::{Read, Write};

use termion::color;

use crate::buffer::{self, Buffer};

pub struct Context<R> {
    buffer: Buffer<R>,
    width: usize,
    height: usize,
    offset: usize,
    cmd_line_kind: CommandLineKind,
}

#[derive(Clone, PartialEq, Eq)]
enum CommandLineKind {
    Filename(String),
    Normal,
    End,
    Search(SearchLineData),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CommandLineKindExt {
    Normal,
    Search,
}

impl From<&CommandLineKind> for CommandLineKindExt {
    fn from(kind: &CommandLineKind) -> Self {
        match kind {
            CommandLineKind::Filename(_) | CommandLineKind::Normal | CommandLineKind::End => {
                CommandLineKindExt::Normal
            }
            CommandLineKind::Search(_) => CommandLineKindExt::Search,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct SearchLineData {
    search: String,
    pos: usize,
}

impl SearchLineData {
    fn new() -> Self {
        Self {
            search: String::new(),
            pos: 0,
        }
    }

    fn insert_char(&mut self, ch: char) {
        self.search.insert(self.pos, ch);
        self.pos += 1;
    }

    // Has semantics of backspace
    fn erase_char(&mut self) {
        if self.pos > 0 {
            self.search.remove(self.pos - 1);
            self.pos -= 1;
        }
    }

    // Has semantics of delete
    fn delete_char(&mut self) {
        if self.pos < self.search.len() {
            self.search.remove(self.pos);
        }
    }

    fn cursor_left(&mut self) {
        self.pos = self.pos.saturating_sub(1);
    }

    fn cursor_right(&mut self) {
        if self.pos < self.search.len() {
            self.pos += 1;
        }
    }
}

impl<R> Context<R> {
    pub fn cmd_line_kind(&self) -> CommandLineKindExt {
        (&self.cmd_line_kind).into()
    }

    pub fn switch_to_search_mode(&mut self) {
        self.cmd_line_kind = CommandLineKind::Search(SearchLineData::new());
    }

    pub fn switch_to_normal_mode(&mut self) {
        self.cmd_line_kind = CommandLineKind::Normal;
    }

    pub fn search_push_char(&mut self, ch: char) {
        if let CommandLineKind::Search(sld) = &mut self.cmd_line_kind {
            sld.insert_char(ch);
        } else {
            unreachable!("Expected to be in search mode");
        }
    }

    // For backspace
    pub fn search_erase_char(&mut self) {
        if let CommandLineKind::Search(sld) = &mut self.cmd_line_kind {
            sld.erase_char();
        } else {
            unreachable!("Expected to be in search mode");
        }
    }

    // For delete
    pub fn search_delete_char(&mut self) {
        if let CommandLineKind::Search(sld) = &mut self.cmd_line_kind {
            sld.delete_char();
        } else {
            unreachable!("Expected to be in search mode");
        }
    }

    pub fn search_cursor_left(&mut self) {
        if let CommandLineKind::Search(sld) = &mut self.cmd_line_kind {
            sld.cursor_left();
        } else {
            unreachable!("Expected to be in search mode");
        }
    }

    pub fn search_cursor_right(&mut self) {
        if let CommandLineKind::Search(sld) = &mut self.cmd_line_kind {
            sld.cursor_right();
        } else {
            unreachable!("Expected to be in search mode");
        }
    }

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
        write!(writer, "{}", termion::clear::CurrentLine)?;
        match &self.cmd_line_kind {
            CommandLineKind::Filename(name) => write!(
                writer,
                "{}{}{}{}{}",
                color::Fg(color::Black),
                color::Bg(color::LightWhite),
                name,
                color::Fg(color::Reset),
                color::Bg(color::Reset),
            )?,
            CommandLineKind::Normal => write!(writer, ":")?,
            CommandLineKind::End => write!(
                writer,
                "{}{}(END){}{}",
                color::Fg(color::Black),
                color::Bg(color::LightWhite),
                color::Fg(color::Reset),
                color::Bg(color::Reset),
            )?,
            CommandLineKind::Search(sld) => {
                write!(writer, "/{}", sld.search)?;
                write!(
                    writer,
                    "{}",
                    termion::cursor::Goto((sld.pos + 2) as u16, self.height as u16)
                )?;
            }
        }
        writer.flush()?;
        Ok(())
    }
}

impl<R> Context<R>
where
    R: Read,
{
    pub fn new(width: u16, height: u16, reader: R, filename: Option<String>) -> Self {
        debug!("Width: {}, Height: {}", width, height);
        Self {
            buffer: Buffer::new(reader),
            width: width as usize,
            height: height as usize,
            offset: 0,
            cmd_line_kind: filename.map_or(CommandLineKind::Normal, |name| {
                CommandLineKind::Filename(name)
            }),
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
        let old_kind = self.cmd_line_kind.clone();
        self.offset += lines;
        self.cmd_line_kind = CommandLineKind::Normal;
        while self.buffer.len() - self.offset < self.height - 1 {
            let line = self.buffer.append_line()?;
            if line.is_none() {
                self.offset = self.buffer.len() - self.height + 1;
                self.cmd_line_kind = CommandLineKind::End;
                debug!("Hit end");
                break;
            }
        }
        Ok(old_offset != self.offset || old_kind != self.cmd_line_kind)
    }

    pub fn scroll_up(&mut self, lines: usize) -> anyhow::Result<bool> {
        let old_offset = self.offset;
        let old_kind = self.cmd_line_kind.clone();
        self.offset = self.offset.saturating_sub(lines);
        self.cmd_line_kind = CommandLineKind::Normal;
        Ok(old_offset != self.offset || old_kind != self.cmd_line_kind)
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
