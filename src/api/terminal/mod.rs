use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, List, ListState, Paragraph},
};

pub(crate) struct TerminalResult {
    pub confirmed: bool,
    pub items: Vec<String>,
}

pub(crate) struct Terminal {
    items: Vec<String>,
    title: String,
    list_state: ListState,
    quit: bool,
    confirmed: bool,
}

impl Terminal {
    pub fn new_inline(items: Vec<String>, title: &str) -> Self {
        let mut list_state = ListState::default();
        list_state.select_first();

        Self {
            items,
            title: title.to_string(),
            list_state,
            quit: false,
            confirmed: false,
        }
    }

    pub fn run(mut self) -> crate::Result<TerminalResult> {
        let mut terminal = ratatui::init();
        let result = self.event_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn event_loop(&mut self, terminal: &mut DefaultTerminal) -> crate::Result<TerminalResult> {
        while !self.quit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_event()?;
        }

        Ok(TerminalResult {
            confirmed: self.confirmed,
            items: std::mem::take(&mut self.items),
        })
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let [list_area, hint_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
                .areas(frame.area());

        let list = List::new(self.items.clone())
            .block(Block::bordered().title(self.title.as_str()))
            .highlight_style(Style::new().reversed())
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, list_area, &mut self.list_state);

        let hints = Paragraph::new("[s/Enter] Save  [q/Esc] Quit")
            .centered()
            .dim();

        frame.render_widget(hints, hint_area);
    }

    fn handle_event(&mut self) -> crate::Result<()> {
        let Event::Key(key) = event::read()? else {
            return Ok(());
        };

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.list_state.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.list_state.select_last(),
            KeyCode::Char('s') | KeyCode::Enter => {
                self.confirmed = true;
                self.quit = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
            }
            _ => {}
        }

        Ok(())
    }
}
