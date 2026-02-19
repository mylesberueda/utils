mod confirm;
mod select_list;

#[allow(unused_imports)]
pub(crate) use confirm::ConfirmPrompt;
pub(crate) use select_list::{SelectList, SelectResult};

use crossterm::{ExecutableCommand, cursor, terminal};
use ratatui::{TerminalOptions, Viewport, prelude::*};
use std::io::stdout;

pub(crate) struct InlineTerminal {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    start_row: u16,
    cleaned_up: bool,
}

impl InlineTerminal {
    pub(crate) fn new(height: u16, width: u16) -> crate::Result<Self> {
        terminal::enable_raw_mode()?;

        for _ in 0..height {
            println!();
        }

        stdout().execute(cursor::MoveUp(height))?;

        let start_row = cursor::position()?.1;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, start_row, width, height)),
            },
        )?;

        Ok(Self {
            terminal,
            start_row,
            cleaned_up: false,
        })
    }

    pub(crate) fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> crate::Result<()> {
        self.terminal.draw(f)?;
        Ok(())
    }

    pub(crate) fn cleanup(&mut self) -> crate::Result<()> {
        if self.cleaned_up {
            return Ok(());
        }
        self.cleaned_up = true;

        terminal::disable_raw_mode()?;
        stdout().execute(cursor::MoveTo(0, self.start_row))?;
        stdout().execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        Ok(())
    }
}

impl Drop for InlineTerminal {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
