use std::fs::File;
use std::io::{Read, Write};

use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::TermRead;

use crate::context::{CommandLineKindExt, Context};

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
        match self.ctx.cmd_line_kind() {
            CommandLineKindExt::Normal => self.handle_normal_key_event(key),
            CommandLineKindExt::Search => self.handle_search_key_event(key),
        }
    }

    fn handle_normal_key_event(&mut self, key: Key) -> anyhow::Result<bool> {
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
            Key::Char(' ') | Key::Ctrl('v') | Key::Char('f') | Key::Ctrl('f') | Key::PageDown => {
                if self.ctx.scroll_down_screen()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            Key::PageUp => {
                if self.ctx.scroll_up_screen()? {
                    self.ctx.write_screen(&mut self.screen)?;
                }
            }
            Key::Char('/') => {
                self.ctx.switch_to_search_mode();
                self.ctx.write_screen(&mut self.screen)?;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_search_key_event(&mut self, key: Key) -> anyhow::Result<bool> {
        match key {
            Key::Char(c) => {
                self.ctx.search_push_char(c);
                self.ctx.write_screen(&mut self.screen)?;
            }
            Key::Esc => {
                self.ctx.switch_to_normal_mode();
                self.ctx.write_screen(&mut self.screen)?;
            }
            Key::Backspace => {
                self.ctx.search_pop_char();
                self.ctx.write_screen(&mut self.screen)?;
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
