use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::Paragraph};

use super::InlineTerminal;

/// A reusable Yes/No confirmation prompt rendered inline.
///
/// Displays a header, body lines, and a left/right Yes/No selector.
/// Returns `true` if confirmed, `false` if cancelled.
#[allow(dead_code)]
pub(crate) struct ConfirmPrompt {
    header: String,
    lines: Vec<Line<'static>>,
}

impl ConfirmPrompt {
    pub(crate) fn new(header: impl Into<String>, lines: Vec<Line<'static>>) -> Self {
        Self {
            header: header.into(),
            lines,
        }
    }

    /// Run the interactive confirmation loop.
    pub(crate) fn run(&mut self, terminal: &mut InlineTerminal) -> crate::Result<bool> {
        // Default to "No" (safe choice)
        let mut selected_yes = false;

        loop {
            let header = &self.header;
            let body_lines = &self.lines;

            terminal.draw(|f| {
                let mut lines: Vec<Line> = Vec::new();

                // Header
                lines.push(Line::from(vec![Span::styled(
                    header.as_str(),
                    Style::default().fg(Color::Yellow).bold(),
                )]));
                lines.push(Line::from(""));

                // Body content
                lines.extend(body_lines.iter().cloned());

                // Confirmation selector
                lines.push(Line::from(""));

                let yes_style = if selected_yes {
                    Style::default().bg(Color::DarkGray).fg(Color::Green).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let no_style = if !selected_yes {
                    Style::default().bg(Color::DarkGray).fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(" Yes, confirm ", yes_style),
                    Span::raw("  "),
                    Span::styled(" No, cancel ", no_style),
                    Span::styled(
                        "  (←→ select, Enter confirm)",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));

                f.render_widget(Paragraph::new(lines), f.area());
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        KeyCode::Left | KeyCode::Char('h') => selected_yes = true,
                        KeyCode::Right | KeyCode::Char('l') => selected_yes = false,
                        KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            return Ok(false)
                        }
                        KeyCode::Enter => return Ok(selected_yes),
                        _ => {}
                    }
                }
            }
        }
    }
}
