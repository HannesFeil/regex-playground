use ratatui::{
    style::{Style, Stylize as _},
    text::{Line, Span},
};
use regex::Regex;
use regex_syntax::ast;
use tui_input::{Input, InputRequest, InputResponse};

pub struct RegexInput {
    regex_input: Input,
    regex_parsed: Result<ast::Ast, ast::Error>,
    regex_line: Line<'static>,
    regex: Result<Regex, regex::Error>,
}

impl RegexInput {
    pub fn new() -> Self {
        Self {
            regex_input: Input::new("".to_owned()),
            regex_parsed: ast::parse::Parser::new().parse(""),
            regex_line: Line::raw(""),
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
            Ok(ast) => ast::visit(ast, RegexFormatter::new(string)).unwrap(),
            Err(_) => Line::raw(string),
        };
    }

    pub fn input_line(&self) -> &Line<'static> {
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
