use std::fs::File;
use std::io::{Read, Write};

use termion::event::{Event, Key};
use termion::input::TermRead;

use crate::context::Context;

pub struct CommandDispatcher<'a, R, W> {
    ctx: &'a mut Context<R>,
    screen: W,
}

impl<'a, R, W> CommandDispatcher<'a, R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(ctx: &'a mut Context<R>, screen: W) -> Self {
        Self { ctx, screen }
    }

    pub fn handle_events(&mut self, tty: File) -> anyhow::Result<()> {
        for event in tty.events() {
            let event = event?;
            match event {
                Event::Key(key) => {
                    if self.handle_key_event(key)? {
                        break;
                    }
                }
                Event::Mouse(_) => {}
                Event::Unsupported(ev) => info!("Unsupported event: {:?}", ev),
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: Key) -> anyhow::Result<bool> {
        match key {
            Key::Char('q') => return Ok(true),
            Key::Char('j') => {
                if self.ctx.scroll_down()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            Key::Char('k') => {
                if self.ctx.sroll_up()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            _ => {}
        }
        Ok(false)
    }
}