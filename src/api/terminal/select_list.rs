use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{List, ListItem, ListState, Paragraph},
};

use super::InlineTerminal;

pub(crate) enum SelectResult {
    Confirmed,
    Cancelled,
}

struct ConfirmButtons {
    confirm_label: String,
    cancel_label: String,
    selected_confirm: bool,
}

pub(crate) struct SelectList<T> {
    items: Vec<T>,
    state: ListState,
    header: String,
    display_fn: fn(&T) -> String,
    confirm: Option<ConfirmButtons>,
}

impl<T> SelectList<T> {
    pub(crate) fn new(
        items: Vec<T>,
        header: impl Into<String>,
        display_fn: fn(&T) -> String,
    ) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self {
            items,
            state,
            header: header.into(),
            display_fn,
            confirm: None,
        }
    }

    pub(crate) fn with_confirm(
        mut self,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) -> Self {
        self.confirm = Some(ConfirmButtons {
            confirm_label: confirm_label.into(),
            cancel_label: cancel_label.into(),
            selected_confirm: false,
        });
        self
    }

    pub(crate) fn items(&self) -> &[T] {
        &self.items
    }

    pub(crate) fn run(&mut self, terminal: &mut InlineTerminal) -> crate::Result<SelectResult> {
        let has_buttons = self.confirm.is_some();

        loop {
            let header = &self.header;
            let display_fn = self.display_fn;
            let items = &self.items;
            let state = &mut self.state;
            let confirm = &self.confirm;

            terminal.draw(|f| {
                let area = f.area();

                let mut constraints = vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ];

                if has_buttons {
                    constraints.push(Constraint::Length(1));
                    constraints.push(Constraint::Length(1));
                } else {
                    constraints.push(Constraint::Length(1));
                }

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(area);

                let header_line = Line::from(Span::styled(
                    header.as_str(),
                    Style::default().fg(Color::Yellow).bold(),
                ));

                f.render_widget(Paragraph::new(header_line), chunks[0]);
                f.render_widget(Paragraph::new(""), chunks[1]);

                let list_items: Vec<ListItem> = items
                    .iter()
                    .map(|item| ListItem::new(display_fn(item)))
                    .collect();

                let list = List::new(list_items)
                    .highlight_style(Style::default().bg(Color::DarkGray).bold())
                    .highlight_symbol(" > ");

                f.render_stateful_widget(list, chunks[2], state);

                if let Some(buttons) = confirm {
                    f.render_widget(Paragraph::new(""), chunks[3]);

                    let confirm_style = if buttons.selected_confirm {
                        Style::default().bg(Color::DarkGray).fg(Color::Green).bold()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let cancel_style = if !buttons.selected_confirm {
                        Style::default().bg(Color::DarkGray).fg(Color::Red).bold()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let button_line = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!(" {} ", buttons.confirm_label), confirm_style),
                        Span::raw("  "),
                        Span::styled(format!(" {} ", buttons.cancel_label), cancel_style),
                        Span::styled(
                            "  (← → select, Enter confirm)",
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]);
                    f.render_widget(Paragraph::new(button_line), chunks[4]);
                } else {
                    let hint = "  (↑↓ navigate, Enter confirm, Esc cancel)";
                    f.render_widget(
                        Paragraph::new(Span::styled(hint, Style::default().fg(Color::DarkGray))),
                        chunks[3],
                    );
                }
            })?;

            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(SelectResult::Cancelled);
                    }

                    KeyCode::Up | KeyCode::Char('k') => {
                        let cursor = self.state.selected().unwrap_or(0);
                        if cursor > 0 {
                            self.state.select(Some(cursor - 1));
                        }
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        let cursor = self.state.selected().unwrap_or(0);
                        if cursor < self.items.len().saturating_sub(1) {
                            self.state.select(Some(cursor + 1));
                        }
                    }

                    KeyCode::Home | KeyCode::Char('g') => {
                        self.state.select(Some(0));
                    }

                    KeyCode::End | KeyCode::Char('G') => {
                        if !self.items.is_empty() {
                            self.state.select(Some(self.items.len() - 1));
                        }
                    }

                    KeyCode::Left | KeyCode::Char('h') if has_buttons => {
                        if let Some(ref mut buttons) = self.confirm {
                            buttons.selected_confirm = true;
                        }
                    }

                    KeyCode::Right | KeyCode::Char('l') if has_buttons => {
                        if let Some(ref mut buttons) = self.confirm {
                            buttons.selected_confirm = false;
                        }
                    }

                    KeyCode::Tab if has_buttons => {
                        if let Some(ref mut buttons) = self.confirm {
                            buttons.selected_confirm = !buttons.selected_confirm;
                        }
                    }

                    KeyCode::Char('y') | KeyCode::Char('Y') if has_buttons => {
                        return Ok(SelectResult::Confirmed);
                    }

                    KeyCode::Char('n') | KeyCode::Char('N') if has_buttons => {
                        return Ok(SelectResult::Cancelled);
                    }

                    KeyCode::Enter => {
                        if has_buttons {
                            let confirmed = self
                                .confirm
                                .as_ref()
                                .map(|b| b.selected_confirm)
                                .unwrap_or(false);
                            return if confirmed {
                                Ok(SelectResult::Confirmed)
                            } else {
                                Ok(SelectResult::Cancelled)
                            };
                        } else {
                            return Ok(SelectResult::Confirmed);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
