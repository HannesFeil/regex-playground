use std::io;

use crossterm::event;
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::tui::Tui;

pub struct App {
    running: bool,
}

impl App {
    pub fn new() -> Self {
        App { running: true }
    }

    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while self.running {
            self.handle_events();
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.size()))?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            event::Event::Key(key_event) => self.handle_key_event(key_event),
            event::Event::Paste(string) => todo!(),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new("Test text woop")
            .centered()
            .block(Block::bordered())
            .render(area, buf);
    }
}
