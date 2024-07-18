use std::{
    collections::BinaryHeap,
    io::{self, stdout, Stdout},
};

use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::debug;
use ratatui::{
    backend::CrosstermBackend,
    style::Style,
    text::{Span, Text},
    Terminal,
};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen, DisableBracketedPaste)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn style_string(string: &str, styles: &[Style]) -> Text<'static> {
    let mut lines = vec![];
    let mut spans = vec![];

    for (index, char) in string.char_indices() {
        match char {
            '\n' => {
                let mut new = vec![];
                std::mem::swap(&mut new, &mut spans);
                lines.push(new);
            }
            _ => {
                spans.push(Span::styled(char.to_string(), styles[index]));
            }
        }
    }

    if !spans.is_empty() {
        lines.push(spans);
    }

    Text::from_iter(lines)
}
