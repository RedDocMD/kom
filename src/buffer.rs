use std::io::Read;

use termion::input::TermRead;

pub struct Buffer<R> {
    lines: Vec<String>,
    reader: R,
}

impl<R> Buffer<R> {
    pub fn new(reader: R) -> Self {
        Self {
            lines: Vec::new(),
            reader,
        }
    }

    pub fn lines(&self, width: usize) -> Lines<'_> {
        Lines {
            lines: &self.lines,
            width,
            curr_line: 0,
            curr_idx: 0,
        }
    }

    pub fn render_line_count(&self, width: usize) -> usize {
        self.lines
            .iter()
            .fold(0, |acc, line| acc + line_width_divisions(line, width))
    }
}

pub fn line_width_divisions(line: &str, width: usize) -> usize {
    let len = line.len();
    if len % width == 0 {
        len / width
    } else {
        len / width + 1
    }
}

impl<R> Buffer<R>
where
    R: Read,
{
    pub fn append_line(&mut self) -> anyhow::Result<Option<&str>> {
        let line = self.reader.read_line()?;
        if let Some(line) = line {
            self.lines.push(line);
            Ok(self.lines.last().map(String::as_str))
        } else {
            Ok(None)
        }
    }
}

pub struct Lines<'a> {
    lines: &'a [String],
    width: usize,
    curr_line: usize,
    curr_idx: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_line == self.lines.len() {
            return None;
        }
        let line = &self.lines[self.curr_line];
        let render_width = usize::min(line.len() - self.curr_idx, self.width);
        let render_str = &line[self.curr_idx..(self.curr_idx + render_width)];
        self.curr_idx += render_width;
        if self.curr_idx == line.len() {
            self.curr_line += 1;
            self.curr_idx = 0;
        }
        Some(render_str)
    }
}
