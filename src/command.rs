use std::fs::File;
use std::io::{Read, Write};

use termion::event::{Event, Key, MouseButton, MouseEvent};
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
                Event::Mouse(ev) => self.handle_mouse_event(ev)?,
                Event::Unsupported(ev) => info!("Unsupported event: {:?}", ev),
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: Key) -> anyhow::Result<bool> {
        match key {
            Key::Char('q') => return Ok(true),
            Key::Char('j') => {
                if self.ctx.scroll_down_line()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            Key::Char('k') => {
                if self.ctx.scroll_up_line()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            Key::Char(' ') | Key::Ctrl('v') | Key::Char('f') | Key::Ctrl('f') => {
                if self.ctx.scroll_down_screen()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_mouse_event(&mut self, mouse_ev: MouseEvent) -> anyhow::Result<()> {
        debug!("Mouse event: {:?}", mouse_ev);
        if let MouseEvent::Press(btn, _, _) = mouse_ev {
            match btn {
                MouseButton::WheelUp => {
                    if self.ctx.scroll_up_line()? {
                        self.ctx.write_screen(&mut self.screen)?;
                    }
                }
                MouseButton::WheelDown => {
                    if self.ctx.scroll_down_line()? {
                        self.ctx.write_screen(&mut self.screen)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
