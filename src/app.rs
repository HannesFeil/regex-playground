use std::io;

use crossterm::event;
use log::trace;
use ratatui::{
    layout::{Constraint, Layout},
    text::ToLine,
    widgets::{Block, Paragraph, Row, Widget},
};
use tui_input::Input;

use crate::tui::Tui;

pub struct App {
    running: bool,
    regex_input: Input,
    data: String,
    focus_state: FocusState,
}

enum FocusState {
    TypingRegex,
}

impl App {
    pub fn new(data: String) -> Self {
        App {
            running: true,
            regex_input: Input::new("awesome regex".to_owned()),
            data,
            focus_state: FocusState::TypingRegex,
        }
    }

    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.size());
                if let Some((cursor_x, cursor_y)) = match self.focus_state {
                    FocusState::TypingRegex => Some((1 + self.regex_input.visual_cursor() as u16, 1)),
                } {
                    frame.set_cursor(cursor_x, cursor_y);
                }
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        trace!("Got event {event:?}");
        let input = tui_input::backend::crossterm::to_input_request(&event);
        trace!("Input would be: {input:?}");
        match event {
            event::Event::Key(key_event) => self.handle_key_event(key_event, input),
            event::Event::Paste(string) => todo!(),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key_event: event::KeyEvent,
        input: Option<tui_input::InputRequest>,
    ) {
        match key_event {
            event::KeyEvent {
                code: event::KeyCode::Char('q'),
                kind: event::KeyEventKind::Press,
                ..
            } => {
                self.running = false;
            }
            _ => {
                if let Some(input) = input {
                    self.regex_input.handle(input);
                }
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [input_area, preview_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(3)]).areas(area);
        Paragraph::new(self.regex_input.to_line())
            .block(Block::bordered())
            .render(input_area, buf);
        Paragraph::new(self.data.as_str())
            .block(Block::bordered())
            .render(preview_area, buf);
    }
}
