use std::io;

use crossterm::event;
use log::trace;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::ToSpan,
    widgets::{Block, Paragraph, Widget},
};
use regex::RegexInput;

mod regex;

use crate::{config::Config, tui::Tui};

pub struct App {
    running: bool,
    focus_state: FocusState,
    data: String,
    regex_input: RegexInput,
    config: Config,
}

enum FocusState {
    TypingRegex,
}

impl App {
    pub fn new(data: String, config: Config) -> Self {
        Self {
            running: true,
            focus_state: FocusState::TypingRegex,
            data,
            regex_input: RegexInput::new(),
            config,
        }
    }

    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.size());
                if let Some((cursor_x, cursor_y)) = match self.focus_state {
                    FocusState::TypingRegex => Some((1 + self.regex_input.cursor_pos() as u16, 1)),
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
                    let response = self.regex_input.handle_input(input);
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
        let [input_area, mut explanation_area, data_area] = Layout::vertical([
            Constraint::Length(3),
            match self.regex_input.regex() {
                Ok(_) => Constraint::Length(3),
                Err(_) => Constraint::Length(1),
            },
            Constraint::Min(3),
        ])
        .areas(area);
        let [data_area, captures_area] =
            Layout::horizontal(Constraint::from_ratios([(1, 2), (1, 2)])).areas(data_area);

        // Display regex input field
        Paragraph::new(self.regex_input.input_line().clone())
            .block(Block::bordered())
            .render(input_area, buf);

        // Display error / explanation
        explanation_area = explanation_area.inner(Margin {
            horizontal: 1,
            vertical: 0,
        });
        match self.regex_input.regex() {
            Ok(_) => Paragraph::new("Nice").render(explanation_area, buf),
            Err(error) => match error {
                ::regex::Error::Syntax(_) => match self.regex_input.ast() {
                    Ok(_) => unreachable!(),
                    Err(error) => {
                        Paragraph::new(error.kind().to_span().style(Style::new().light_red()))
                            .render(explanation_area, buf);
                        let marker_rect = Rect::new(
                            input_area.x + error.span().start.column as u16,
                            input_area.y + 1,
                            (error.span().end.column - error.span().start.column).max(1) as u16,
                            1,
                        );
                        buf.set_style(marker_rect, Style::new().black().on_light_red());
                        if let Some(span) = error.auxiliary_span() {
                            let marker_rect = Rect::new(
                                input_area.x + span.start.column as u16,
                                input_area.y + 1,
                                (span.end.column - span.start.column).max(1) as u16,
                                1,
                            );
                            buf.set_style(marker_rect, Style::new().black().on_light_red());
                        }
                    }
                },
                ::regex::Error::CompiledTooBig(_) => {
                    Paragraph::new("Compiled regex ist too big!".light_red())
                        .render(explanation_area, buf)
                }
                _ => Paragraph::new("Unknown error occurred!".light_red())
                    .render(explanation_area, buf),
            },
        }

        // Display data field
        Paragraph::new(self.data.as_str())
            .block(Block::bordered())
            .render(data_area, buf);

        // Display captures here
        match &self.regex_input.regex() {
            Ok(regex) => {
                Paragraph::new(format!("{:?}", regex.captures(&self.data)))
                    .block(Block::bordered())
                    .render(captures_area, buf);
            }
            Err(error) => {
                Paragraph::new(format!("{:?}", error))
                    .block(Block::bordered())
                    .render(captures_area, buf);
            }
        }
    }
}
