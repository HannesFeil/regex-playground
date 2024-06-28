use std::io;

use crossterm::event;
use log::trace;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use regex::Regex;
use regex_syntax::ast;
use tui_input::Input;

use crate::tui::Tui;

pub struct App {
    running: bool,
    focus_state: FocusState,
    data: String,
    regex_input: Input,
    regex_parsed: Result<ast::Ast, ast::Error>,
    regex_line: Line<'static>,
    regex: Result<Regex, regex::Error>,
}

enum FocusState {
    TypingRegex,
}

impl App {
    pub fn new(data: String) -> Self {
        let mut app = App {
            running: true,
            focus_state: FocusState::TypingRegex,
            data,
            regex_input: Input::new("".to_owned()),
            regex_parsed: ast::parse::Parser::new().parse(""),
            regex_line: Line::raw(""),
            regex: Regex::new(""),
        };

        app.update_regex();

        app
    }

    fn update_regex(&mut self) {
        let string = self.regex_input.value().to_owned();
        self.regex_parsed = ast::parse::Parser::new().parse(&string);
        self.regex = Regex::new(&string);
        self.regex_line = match &self.regex_parsed {
            Ok(ast) => ast::visit(ast, RegexFormatter::new(string)).unwrap(),
            Err(_) => Line::raw(string),
        };
    }

    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.size());
                if let Some((cursor_x, cursor_y)) = match self.focus_state {
                    FocusState::TypingRegex => {
                        Some((1 + self.regex_input.visual_cursor() as u16, 1))
                    }
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
                    let response = self.regex_input.handle(input);
                    if response.is_some_and(|response| response.value) {
                        self.update_regex();
                    }
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
        let [input_area, explanation_area, data_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .areas(area);
        let [data_area, captures_area] =
            Layout::horizontal(Constraint::from_ratios([(1, 2), (1, 2)])).areas(data_area);
        Paragraph::new(self.regex_line.clone())
            .block(Block::bordered())
            .render(input_area, buf);
        Paragraph::new(Line::from_iter(["test", "\n", "lol"]))
            .render(explanation_area, buf);
        Paragraph::new(self.data.as_str())
            .block(Block::bordered())
            .render(data_area, buf);
        match &self.regex {
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

struct RegexFormatter {
    input: String,
    styles: Vec<Style>,
}

impl RegexFormatter {
    fn new(input: String) -> Self {
        let len = input.len();

        RegexFormatter {
            input,
            styles: vec![Style::new(); len],
        }
    }

    fn style_ast(ast: &ast::Ast, style: Style) -> Style {
        match ast {
            ast::Ast::Empty(_) => style.light_magenta(),
            ast::Ast::Flags(flags) => style.light_cyan(),
            ast::Ast::Literal(literal) => style.yellow(),
            ast::Ast::Dot(_) => style.white(),
            ast::Ast::Assertion(assertion) => style.light_red(),
            ast::Ast::ClassUnicode(unicode_class) => style.light_yellow(),
            ast::Ast::ClassPerl(perl_class) => style.magenta(),
            ast::Ast::ClassBracketed(bracket_class) => style.cyan(),
            ast::Ast::Repetition(repetition) => style.light_green(),
            ast::Ast::Group(group) => style.green(),
            ast::Ast::Alternation(alternation) => style.blue(),
            ast::Ast::Concat(concatenation) => style,
        }
    }

    fn style_class_item(ast: &ast::ClassSetItem, style: Style) -> Style {
        match ast {
            ast::ClassSetItem::Empty(_) => style,
            ast::ClassSetItem::Literal(_) => style.yellow(),
            ast::ClassSetItem::Range(_) => style.cyan(),
            ast::ClassSetItem::Ascii(_) => style.blue(),
            ast::ClassSetItem::Unicode(_) => style.light_yellow(),
            ast::ClassSetItem::Perl(_) => style.green(),
            ast::ClassSetItem::Bracketed(_) => style,
            ast::ClassSetItem::Union(_) => style,
        }
    }
}

impl ast::Visitor for RegexFormatter {
    type Output = Line<'static>;
    type Err = String;

    fn finish(self) -> Result<Self::Output, Self::Err> {
        Ok(self
            .input
            .char_indices()
            .map(|(index, char)| Span::styled(char.to_string(), self.styles[index]))
            .collect())
    }

    fn visit_pre(&mut self, ast: &ast::Ast) -> Result<(), Self::Err> {
        let current_styles = &mut self.styles[ast.span().start.offset..ast.span().end.offset];
        current_styles.iter_mut().for_each(|style| {
            *style = RegexFormatter::style_ast(ast, *style);
        });

        Ok(())
    }

    fn visit_class_set_item_pre(&mut self, ast: &ast::ClassSetItem) -> Result<(), Self::Err> {
        let current_styles = &mut self.styles[ast.span().start.offset..ast.span().end.offset];
        current_styles.iter_mut().for_each(|style| {
            *style = RegexFormatter::style_class_item(ast, *style);
        });

        Ok(())
    }
}
