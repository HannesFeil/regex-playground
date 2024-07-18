use ratatui::{
    style::{Style, Stylize as _},
    text::Text,
};
use regex::{Captures, Regex};
use regex_syntax::ast;
use tui_input::{Input, InputRequest, InputResponse};

pub struct RegexInput {
    regex_input: Input,
    regex_parsed: Result<ast::Ast, ast::Error>,
    regex_line: Text<'static>,
    regex: Result<Regex, regex::Error>,
}

impl RegexInput {
    pub fn new() -> Self {
        Self {
            regex_input: Input::new("".to_owned()),
            regex_parsed: ast::parse::Parser::new().parse(""),
            regex_line: Text::from(""),
            regex: Regex::new(""),
        }
    }

    pub fn handle_input(&mut self, request: InputRequest) -> InputResponse {
        self.regex_input.handle(request).inspect(|response| {
            if response.value {
                self.update_regex();
            }
        })
    }

    fn update_regex(&mut self) {
        let string = self.regex_input.value().to_owned();
        self.regex_parsed = ast::parse::Parser::new().parse(&string);
        self.regex = Regex::new(&string);
        self.regex_line = match &self.regex_parsed {
            Ok(ast) => crate::tui::style_string(
                &string,
                &ast::visit(ast, RegexFormatter::new(string.len())).unwrap(),
            ),
            Err(_) => Text::from(string),
        };
    }

    pub fn input_line(&self) -> &Text<'static> {
        &self.regex_line
    }

    pub fn cursor_pos(&self) -> u16 {
        self.regex_input
            .visual_cursor()
            .try_into()
            .expect("Expect input length to fit into u16")
    }

    pub fn regex(&self) -> &Result<Regex, regex::Error> {
        &self.regex
    }

    pub fn ast(&self) -> &Result<ast::Ast, ast::Error> {
        &self.regex_parsed
    }
}

pub fn style_captures<'a>(len: usize, captures: impl Iterator<Item = Captures<'a>>) -> Vec<Style> {
    let mut styles = vec![Style::new(); len];

    for capture in captures {
        let group0 = capture.get(0).unwrap();
        for i in group0.start()..group0.end() {
            styles[i] = styles[i].on_red();
        }
    }

    styles
}

struct RegexFormatter {
    styles: Vec<Style>,
}

impl RegexFormatter {
    fn new(len: usize) -> Self {
        RegexFormatter {
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
    type Output = Vec<Style>;
    type Err = String;

    fn finish(self) -> Result<Self::Output, Self::Err> {
        Ok(self.styles)
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
